//! Workspaces command router.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Workspaces::{
		EnterWorkspace,
		CreateUntitledWorkspace,
		DeleteUntitledWorkspace,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes workspaces commands.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"workspaces:getFolders" | "workspaces:getWorkspaceFolders" | "workspaces:getWorkspace" => {
			dev_log!("workspaces", "{}", command);

			Some(EnterWorkspace::Fn(RunTime.clone()).await)
		},

		"workspaces:addFolder" | "workspaces:addWorkspaceFolders" => {
			dev_log!("workspaces", "{}", command);

			Some(CreateUntitledWorkspace::Fn(RunTime.clone(), Arguments).await)
		},

		// Note: DeleteUntitledWorkspace takes app_handle in its original inline form
		// but the DeadWorkspace variant removed it. This router uses Runtime-only.
		"workspaces:removeFolder" | "workspaces:removeWorkspaceFolders" => {
			dev_log!("workspaces", "{}", command);

			Some(DeleteUntitledWorkspace::Fn(RunTime.clone(), Arguments).await)
		},

		"workspaces:getName" => {
			dev_log!("workspaces", "{}", command);

			Some(EnterWorkspace::Fn(RunTime.clone()).await)
		},

		_ => None,
	}
}
