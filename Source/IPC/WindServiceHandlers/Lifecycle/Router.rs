//! Lifecycle command router.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::{
		Lifecycle::LifecycleAdvancePhase::Fn as LifecycleAdvancePhase,
		UI::{
			LifecycleGetPhase::Fn as LifecycleGetPhase,
			LifecycleRequestShutdown::Fn as LifecycleRequestShutdown,
			LifecycleWhenPhase::Fn as LifecycleWhenPhase,
		},
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes lifecycle commands. Returns Some(result) for handled commands,
/// None otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	ApplicationHandle:tauri::AppHandle,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"lifecycle:getPhase" => Some(LifecycleGetPhase(RunTime).await),

		"lifecycle:whenPhase" => {
			dev_log!("lifecycle", "{}", command);

			Some(LifecycleWhenPhase(RunTime, Arguments).await)
		},

		"lifecycle:requestShutdown" => {
			dev_log!("lifecycle", "{}", command);

			Some(LifecycleRequestShutdown(ApplicationHandle).await)
		},

		"lifecycle:advancePhase" | "lifecycle:setPhase" => {
			dev_log!("lifecycle", "{}", command);

			Some(LifecycleAdvancePhase(RunTime, ApplicationHandle, Arguments).await)
		},

		_ => None,
	}
}
