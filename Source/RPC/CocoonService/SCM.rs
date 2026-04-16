#![allow(non_snake_case)]
//! SCM (Source Control Management) domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_scm_provider, update_scm_group, git_exec.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;
use crate::dev_log;
use crate::Vine::Generated::{
	Empty, GitExecRequest, GitExecResponse, RegisterScmProviderRequest,
	UpdateScmGroupRequest,
};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

pub async fn RegisterScmProvider(
	Service:&CocoonServiceImpl,
	req:RegisterScmProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering SCM provider: {}", req.scm_id);

	let Handle = req.scm_id.as_bytes().iter().fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
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

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://scm/register",
		json!({ "scmId": req.scm_id, "extensionId": req.extension_id }),
	);

	Ok(Response::new(Empty {}))
}

pub async fn UpdateScmGroup(
	Service:&CocoonServiceImpl,
	req:UpdateScmGroupRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] update_scm_group: provider={} group={}", req.provider_id, req.group_id);

	let ResourceStates:Vec<serde_json::Value> = req.resource_states.iter().map(|Rs| {
		json!({
			"uri": Rs.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
			"decorations": Rs.decorations,
		})
	}).collect();

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://scm/updateGroup",
		json!({
			"providerId": req.provider_id,
			"groupId": req.group_id,
			"resourceStates": ResourceStates,
		}),
	);

	Ok(Response::new(Empty {}))
}

pub async fn GitExec(
	Service:&CocoonServiceImpl,
	req:GitExecRequest,
) -> Result<Response<GitExecResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] git_exec: {}", req.args.join(" "));

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
			Status::internal(format!("git_exec: failed to spawn git: {}", Error))
		})?;

	let ExitCode = Output.status.code().unwrap_or(-1);
	dev_log!("cocoon",
		"[CocoonService] git_exec exit={} stdout={} bytes stderr={} bytes",
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
