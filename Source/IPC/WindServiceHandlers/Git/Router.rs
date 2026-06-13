//! Git command router — delegates all `git:*` IPC commands to
//! the corresponding atom handlers.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Git::{
		HandleCancel::Fn as HandleCancel,
		HandleCheckout::Fn as HandleCheckout,
		HandleClone::Fn as HandleClone,
		HandleExec::Fn as HandleExec,
		HandleFetch::Fn as HandleFetch,
		HandleIsAvailable::Fn as HandleIsAvailable,
		HandlePull::Fn as HandlePull,
		HandleRevListCount::Fn as HandleRevListCount,
		HandleRevParse::Fn as HandleRevParse,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes git commands. Returns Some(result) for handled commands,
/// None otherwise (caller falls through to next dispatch arm).
pub(crate) async fn route(
	ApplicationHandle:tauri::AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	let _ = ApplicationHandle;

	let _ = RunTime;

	match command {
		"git:exec" => {
			dev_log!("git", "git:exec");

			Some(HandleExec(Arguments).await)
		},

		"git:clone" => {
			dev_log!("git", "git:clone");

			Some(HandleClone(Arguments).await)
		},

		"git:pull" => {
			dev_log!("git", "git:pull");

			Some(HandlePull(Arguments).await)
		},

		"git:checkout" => {
			dev_log!("git", "git:checkout");

			Some(HandleCheckout(Arguments).await)
		},

		"git:revParse" => {
			dev_log!("git", "git:revParse");

			Some(HandleRevParse(Arguments).await)
		},

		"git:fetch" => {
			dev_log!("git", "git:fetch");

			Some(HandleFetch(Arguments).await)
		},

		"git:revListCount" => {
			dev_log!("git", "git:revListCount");

			Some(HandleRevListCount(Arguments).await)
		},

		"git:cancel" => {
			dev_log!("git", "git:cancel");

			Some(HandleCancel(Arguments).await)
		},

		"git:isAvailable" => {
			dev_log!("git", "git:isAvailable");

			Some(HandleIsAvailable(Arguments).await)
		},

		_ => None,
	}
}
