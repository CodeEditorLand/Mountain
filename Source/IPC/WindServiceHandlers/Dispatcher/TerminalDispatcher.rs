//! Terminal command dispatcher.

<<<<<<< HEAD
use serde_json::Value;
=======
use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};
use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::{
	Terminal::{
		AttachToProcess::Fn as AttachToProcess,
		DetachFromProcess::Fn as DetachFromProcess,
		LocalPTYCreateProcess::Fn as LocalPTYCreateProcess,
		LocalPTYFreePortKillProcess::Fn as LocalPTYFreePortKillProcess,
		LocalPTYGetDefaultShell::Fn as LocalPTYGetDefaultShell,
		LocalPTYGetEnvironment::Fn as LocalPTYGetEnvironment,
		LocalPTYGetProfiles::Fn as LocalPTYGetProfiles,
		LocalPTYResize::Fn as LocalPTYResize,
		ReviveTerminalProcesses::Fn as ReviveTerminalProcesses,
		SerializeTerminalState::Fn as SerializeTerminalState,
		TerminalCreate::Fn as TerminalCreate,
		TerminalDispose::Fn as TerminalDispose,
		TerminalHide::Fn as TerminalHide,
		TerminalSendText::Fn as TerminalSendText,
		TerminalShow::Fn as TerminalShow,
	},
	Utilities::JsonValueHelpers::arg_val,
};
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

use crate::Terminal::{
	AttachToProcess::Fn as AttachToProcess,
	DetachFromProcess::Fn as DetachFromProcess,
	LocalPTYCreateProcess::Fn as LocalPTYCreateProcess,
	LocalPTYFreePortKillProcess::Fn as LocalPTYFreePortKillProcess,
	LocalPTYGetDefaultShell::Fn as LocalPTYGetDefaultShell,
	LocalPTYGetEnvironment::Fn as LocalPTYGetEnvironment,
	LocalPTYGetProfiles::Fn as LocalPTYGetProfiles,
	LocalPTYResize::Fn as LocalPTYResize,
	ReviveTerminalProcesses::Fn as ReviveTerminalProcesses,
	SerializeTerminalState::Fn as SerializeTerminalState,
	TerminalCreate::Fn as TerminalCreate,
	TerminalDispose::Fn as TerminalDispose,
	TerminalHide::Fn as TerminalHide,
	TerminalSendText::Fn as TerminalSendText,
	TerminalShow::Fn as TerminalShow,
};

/// Dispatches terminal commands.
pub async fn dispatch_terminal(
	app_handle:&tauri::AppHandle,

<<<<<<< HEAD
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"terminal:create" => TerminalCreate(runtime.clone(), arguments).await,

		"terminal:sendText" => TerminalSendText(runtime.clone(), arguments).await,

		"terminal:dispose" => TerminalDispose(runtime.clone(), arguments).await,

		"terminal:show" => TerminalShow(runtime.clone(), arguments).await,

		"terminal:hide" => TerminalHide(runtime.clone(), arguments).await,

		"localPty:getProfiles" => LocalPTYGetProfiles().await,

		"localPty:getDefaultSystemShell" => LocalPTYGetDefaultShell().await,

		"localPty:getTerminalLayoutInfo" => {
<<<<<<< HEAD
			// Storage query
			let provider:std::sync::Arc<dyn crate::CommonLibrary::Storage::StorageProvider::StorageProvider> =
				runtime.Environment.Require();

			match provider.GetStorageValue(true, "terminal:layoutInfo".to_string()).await {
=======
			let provider:std::sync::Arc<dyn StorageProvider> = runtime.Environment.Require();

			match provider.GetStorageValue(true, "terminal:layoutInfo").await {
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
				Ok(Some(stored)) => Ok(stored),

				Ok(None) => Ok(Value::Null),

				Err(e) => {
					crate::dev_log!("terminal", "warn: [getTerminalLayoutInfo] storage read failed: {}", e);

					Ok(Value::Null)
				},
			}
		},

		"localPty:setTerminalLayoutInfo" => {
<<<<<<< HEAD
			let provider:std::sync::Arc<dyn crate::CommonLibrary::Storage::StorageProvider::StorageProvider> =
				runtime.Environment.Require();

			let payload = crate::Utilities::JsonValueHelpers::arg_val(&arguments, 0);
=======
			let provider:std::sync::Arc<dyn StorageProvider> = runtime.Environment.Require();

			let payload = arg_val(&arguments, 0);
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

			let _ = provider
				.UpdateStorageValue(true, "terminal:layoutInfo".to_string(), Some(payload))
				.await;

			Ok(Value::Null)
		},

		"localPty:getPerformanceMarks" => Ok(Value::Array(Vec::new())),

		"localPty:reduceConnectionGraceTime" => Ok(Value::Null),

		"localPty:listProcesses" => Ok(Value::Array(Vec::new())),

		"localPty:getEnvironment" => LocalPTYGetEnvironment().await,

		"localPty:getLatency" => Ok(Value::Array(Vec::new())),

		"localPty:spawn" => TerminalCreate(runtime.clone(), arguments).await,

		"localPty:createProcess" => LocalPTYCreateProcess(runtime.clone(), arguments).await,

		"localPty:start" => Ok(Value::Null),

		"localPty:input" | "localPty:write" => TerminalSendText(runtime.clone(), arguments).await,

		"localPty:shutdown" | "localPty:dispose" => TerminalDispose(runtime.clone(), arguments).await,

		"localPty:resize" => LocalPTYResize(runtime.clone(), arguments).await,

		"localPty:acknowledgeDataEvent" => Ok(Value::Null),

		"localPty:getBackendOS" => Ok(Value::Null),

		"localPty:refreshProperty" => Ok(Value::Null),

		"localPty:updateProperty" => Ok(Value::Null),

		"localPty:freePortKillProcess" => LocalPTYFreePortKillProcess(arguments).await,

		"localPty:serializeTerminalState" => SerializeTerminalState(runtime.clone()).await,

		"localPty:reviveTerminalProcesses" => ReviveTerminalProcesses(runtime.clone(), arguments).await,

		"localPty:getRevivedPtyNewId" => {
			let new_id = runtime.Environment.ApplicationState.GetNextTerminalIdentifier();

			crate::dev_log!("terminal", "localPty:getRevivedPtyNewId id={}", new_id);

			Ok(Value::Null)
		},

		"localPty:attachToProcess" => AttachToProcess(runtime.clone(), arguments).await,

		"localPty:detachFromProcess" => DetachFromProcess(runtime.clone(), arguments).await,

		"localPty:setActive" => {
<<<<<<< HEAD
			// Forward to Cocoon
=======
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptActiveTerminalChanged".to_string(),
				arguments.first().cloned().unwrap_or(Value::Null),
			)
			.await;

			Ok(Value::Null)
		},

		"localPty:setShellIntegrationActive" => {
			let term_id = arguments.first().and_then(Value::as_i64).unwrap_or(0) as u64;

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptTerminalShellIntegrationActivated".to_string(),
				json!({ "id": term_id }),
			)
			.await;

			Ok(Value::Null)
		},

		"localPty:setInteracted" => {
<<<<<<< HEAD
			let payload = crate::Utilities::JsonValueHelpers::arg_val(&arguments, 0);
=======
			let payload = arg_val(&arguments, 0);
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptTerminalStateChanged".to_string(),
				payload,
			)
			.await;

			Ok(Value::Null)
		},

		"localPty:setCwd" => {
			let term_id = arguments.first().and_then(Value::as_i64).unwrap_or(0) as u64;

			if let Some(cwd) = arguments.get(1).and_then(Value::as_str) {
				let _ = crate::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"$acceptTerminalCwdChange".to_string(),
					json!({ "id": term_id, "cwd": cwd }),
				)
				.await;
			}

			Ok(Value::Null)
		},

		"localPty:processBinary" => TerminalSendText(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown terminal command: {}", command)),
	}
}
