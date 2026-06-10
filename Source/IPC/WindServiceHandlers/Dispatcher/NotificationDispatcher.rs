//! Notification command dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::UI::{
=======
use crate::IPC::WindServiceHandlers::UI::{
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
	NotificationEndProgress::Fn as NotificationEndProgress,
	NotificationShow::Fn as NotificationShow,
	NotificationShowProgress::Fn as NotificationShowProgress,
	NotificationUpdateProgress::Fn as NotificationUpdateProgress,
};

/// Dispatches notification commands.
///
/// Handled commands:
/// - `notification:show`
/// - `notification:showProgress`
/// - `notification:updateProgress`
/// - `notification:endProgress`
pub async fn dispatch_notification(
	app_handle:&tauri::AppHandle,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"notification:show" => NotificationShow(app_handle.clone(), arguments).await,

		"notification:showProgress" => NotificationShowProgress(app_handle.clone(), arguments).await,

		"notification:updateProgress" => NotificationUpdateProgress(app_handle.clone(), arguments).await,

		"notification:endProgress" => NotificationEndProgress(app_handle.clone(), arguments).await,

		_ => Err(format!("Unknown notification command: {}", command)),
	}
}
