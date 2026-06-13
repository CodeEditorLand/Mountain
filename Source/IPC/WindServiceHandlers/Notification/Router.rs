//! Notification command router.

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::UI::{
		NotificationEndProgress::Fn as NotificationEndProgress,
		NotificationShow::Fn as NotificationShow,
		NotificationShowProgress::Fn as NotificationShowProgress,
		NotificationUpdateProgress::Fn as NotificationUpdateProgress,
	},
	dev_log,
};

/// Routes notification commands.
pub(crate) async fn route(
	ApplicationHandle:tauri::AppHandle,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"notification:show" => {
			dev_log!("notification", "{}", command);

			Some(NotificationShow(ApplicationHandle.clone(), Arguments).await)
		},

		"notification:showProgress" => {
			dev_log!("notification", "{}", command);

			Some(NotificationShowProgress(ApplicationHandle.clone(), Arguments).await)
		},

		"notification:updateProgress" => {
			dev_log!("notification", "{}", command);

			Some(NotificationUpdateProgress(ApplicationHandle.clone(), Arguments).await)
		},

		"notification:endProgress" => {
			dev_log!("notification", "{}", command);

			Some(NotificationEndProgress(ApplicationHandle.clone(), Arguments).await)
		},

		_ => None,
	}
}
