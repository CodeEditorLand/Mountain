#![allow(non_snake_case)]
//! SCM (Source Control Management) domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_scm_provider, update_scm_group, git_exec.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use super::CocoonServiceImpl;
use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Vine::Generated::{Empty, GitExecRequest, GitExecResponse, RegisterScmProviderRequest, UpdateScmGroupRequest},
	dev_log,
};

pub async fn RegisterScmProvider(
	Service:&CocoonServiceImpl,
	req:RegisterScmProviderRequest,
) -> Result<Response<Empty>, Status> {
	use CommonLibrary::SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider;
	dev_log!("cocoon", "[CocoonService] Registering SCM provider: {}", req.scm_id);

	// Keep the existing ProviderRegistration bookkeeping so language
	// feature dispatch can look up the scm provider by handle…
	let Handle = req
		.scm_id
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
	let dto = ProviderRegistrationDTO {
		Handle,
		ProviderType:ProviderType::SourceControl,
		Selector:json!([{ "scmId": req.scm_id }]),
		SideCarIdentifier:"cocoon-main".to_string(),
		ExtensionIdentifier:json!(req.extension_id),
		Options:Some(json!({ "scmId": req.scm_id })),
	};
	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, dto);

	// …and additionally route through the
	// `SourceControlManagementProvider` trait so the SCM state is
	// materialised in `ApplicationState::SourceControl` (the thing
	// Sky's SCM view binds to). The prior direct Sky emit bypassed
	// state tracking, so SCM providers registered by gitlens /
	// svn etc. never appeared in the SCM view until a
	// `UpdateScmGroup` call landed - and even then the group-less
	// header row never materialised.
	let CreateData = json!({
		"id": req.scm_id,
		"label": req.scm_id,
		"rootUri": null,
		"extensionId": req.extension_id,
	});
	if let Err(Error) = Service.environment.CreateSourceControl(CreateData).await {
		dev_log!(
			"cocoon",
			"warn: [CocoonService] CreateSourceControl trait failed ({}); falling back to Sky emit",
			Error
		);
		let _ = Service.environment.ApplicationHandle.emit(
			"sky://scm/register",
			json!({ "scmId": req.scm_id, "extensionId": req.extension_id }),
		);
	}

	Ok(Response::new(Empty {}))
}

pub async fn UpdateScmGroup(Service:&CocoonServiceImpl, req:UpdateScmGroupRequest) -> Result<Response<Empty>, Status> {
	use CommonLibrary::SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider;
	dev_log!(
		"cocoon",
		"[CocoonService] update_scm_group: provider={} group={}",
		req.provider_id,
		req.group_id
	);

	let ResourceStates:Vec<serde_json::Value> = req
		.resource_states
		.iter()
		.map(|Rs| {
			json!({
				"uri": Rs.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
				"decorations": Rs.decorations,
			})
		})
		.collect();

	// Re-derive the provider handle from `provider_id` (same hash
	// RegisterScmProvider used) so the trait can locate the
	// already-registered provider. `UpdateSourceControlGroup`
	// mutates the group in state *and* emits the UI event -
	// downstream Sky components get a deduplicated payload with
	// group metadata instead of a bare list of URIs.
	let ProviderHandle = req
		.provider_id
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
	let GroupData = json!({
		"groupId": req.group_id,
		"label": req.group_id,
		"resourceStates": ResourceStates,
	});
	if let Err(Error) = Service.environment.UpdateSourceControlGroup(ProviderHandle, GroupData).await {
		dev_log!(
			"cocoon",
			"warn: [CocoonService] UpdateSourceControlGroup trait failed ({}); falling back to Sky emit",
			Error
		);
		let _ = Service.environment.ApplicationHandle.emit(
			"sky://scm/updateGroup",
			json!({
				"providerId": req.provider_id,
				"groupId": req.group_id,
				"resourceStates": ResourceStates,
			}),
		);
	}

	Ok(Response::new(Empty {}))
}

pub async fn GitExec(Service:&CocoonServiceImpl, req:GitExecRequest) -> Result<Response<GitExecResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] git_exec: {}", req.args.join(" "));
	dev_log!(
		"git",
		"[Git] exec-begin cwd={} args=[{}]",
		if req.repository_path.is_empty() {
			"<cwd>".to_string()
		} else {
			req.repository_path.clone()
		},
		req.args.join(" ")
	);

	let WorkingDir = if req.repository_path.is_empty() {
		std::env::current_dir().unwrap_or_default()
	} else {
		std::path::PathBuf::from(&req.repository_path)
	};

	let Output = tokio::process::Command::new("git")
		.args(&req.args)
		.current_dir(&WorkingDir)
		.output()
		.await
		.map_err(|Error| {
			dev_log!("cocoon", "error: [CocoonService] git_exec failed to spawn: {}", Error);
			dev_log!(
				"git",
				"[Git] exec-spawn-fail cwd={:?} args=[{}] error={}",
				WorkingDir,
				req.args.join(" "),
				Error
			);
			Status::internal(format!("git_exec: failed to spawn git: {}", Error))
		})?;

	let ExitCode = Output.status.code().unwrap_or(-1);
	dev_log!(
		"cocoon",
		"[CocoonService] git_exec exit={} stdout={} bytes stderr={} bytes",
		ExitCode,
		Output.stdout.len(),
		Output.stderr.len()
	);
	dev_log!(
		"git",
		"[Git] exec-done args=[{}] exit={} stdout={} stderr={}",
		req.args.join(" "),
		ExitCode,
		Output.stdout.len(),
		Output.stderr.len()
	);

	// Combine stdout lines into repeated string output; prepend stderr lines
	// with "stderr: " prefix so extension can differentiate them.
	let StdoutStr = String::from_utf8_lossy(&Output.stdout);
	let StderrStr = String::from_utf8_lossy(&Output.stderr);
	let mut OutputLines:Vec<String> = StdoutStr.lines().map(|L| L.to_string()).collect();
	for Line in StderrStr.lines() {
		OutputLines.push(format!("stderr: {}", Line));
	}

	Ok(Response::new(GitExecResponse { output:OutputLines, exit_code:ExitCode }))
}
