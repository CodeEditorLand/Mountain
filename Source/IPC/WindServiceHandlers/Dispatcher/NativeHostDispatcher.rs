//! NativeHost command dispatcher - Part 1: Dialogs and Environment.

use serde_json::{Value, json};

<<<<<<< HEAD
use crate::NativeHost::{
	GetColorScheme::Fn as NativeGetColorScheme,
	GetEnvironmentPaths::Fn as NativeGetEnvironmentPaths,
	GetOSProperties::Fn as NativeOSProperties,
	GetOSStatistics::Fn as NativeOSStatistics,
	IsFullscreen::Fn as NativeIsFullscreen,
	IsMaximized::Fn as NativeIsMaximized,
=======
use crate::IPC::WindServiceHandlers::NativeHost::{
	GetColorScheme::Fn as NativeGetColorScheme,
	GetEnvironmentPaths::Fn as NativeGetEnvironmentPaths,
	IsFullscreen::Fn as NativeIsFullscreen,
	IsMaximized::Fn as NativeIsMaximized,
	OSProperties::Fn as NativeOSProperties,
	OSStatistics::Fn as NativeOSStatistics,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
	PickFolder::Fn as NativePickFolder,
	ShowMessageBox::Fn as NativeShowMessageBox,
	ShowOpenDialog::Fn as NativeShowOpenDialog,
	ShowSaveDialog::Fn as NativeShowSaveDialog,
	ShowSaveDialogUI::Fn as UserInterfaceShowSaveDialog,
};

/// Dispatches native host dialog and environment commands.
pub async fn dispatch_native_host_dialogs(
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
		"nativeHost:pickFolderAndOpen"
		| "nativeHost:pickFileAndOpen"
		| "nativeHost:pickFileFolderAndOpen"
		| "nativeHost:pickWorkspaceAndOpen" => NativePickFolder(app_handle.clone(), arguments).await,

		"nativeHost:showOpenDialog" => NativeShowOpenDialog(app_handle.clone(), arguments).await,

		"UserInterface.ShowOpenDialog" => {
			match NativeShowOpenDialog(app_handle.clone(), arguments).await {
				Ok(response) => {
					let paths = response
						.get("filePaths")
						.and_then(|v| v.as_array())
						.cloned()
						.unwrap_or_default();

					Ok(Value::Array(paths))
				},

				Err(e) => Err(e),
			}
		},

		"nativeHost:showSaveDialog" => NativeShowSaveDialog(app_handle.clone(), arguments).await,

		"UserInterface.ShowSaveDialog" => UserInterfaceShowSaveDialog(app_handle.clone(), arguments).await,

		"nativeHost:showMessageBox" => NativeShowMessageBox(app_handle.clone(), arguments).await,

		"nativeHost:getEnvironmentPaths" => NativeGetEnvironmentPaths(app_handle.clone()).await,

		"nativeHost:getOSColorScheme" => {
			crate::dev_log!("nativehost", "nativeHost:getOSColorScheme");

			NativeGetColorScheme().await
		},

		"nativeHost:getOSProperties" => {
			crate::dev_log!("nativehost", "nativeHost:getOSProperties");

			NativeOSProperties().await
		},

		"nativeHost:getOSStatistics" => {
			crate::dev_log!("nativehost", "nativeHost:getOSStatistics");

			NativeOSStatistics().await
		},

		"nativeHost:getOSVirtualMachineHint" => {
			crate::dev_log!("nativehost", "nativeHost:getOSVirtualMachineHint");

			Ok(json!(0))
		},

		"nativeHost:isFullScreen" => {
			crate::dev_log!("window", "nativeHost:isFullScreen");

			NativeIsFullscreen(app_handle.clone()).await
		},

		"nativeHost:isMaximized" => {
			crate::dev_log!("window", "nativeHost:isMaximized");

			NativeIsMaximized(app_handle.clone()).await
		},

		_ => Err(format!("Unknown native host dialog command: {}", command)),
	}
}
