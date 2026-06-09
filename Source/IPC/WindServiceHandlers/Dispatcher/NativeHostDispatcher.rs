//! NativeHost command dispatcher - Part 1: Dialogs and Environment.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::NativeHost::{
	GetColorScheme::Fn as NativeGetColorScheme,
	GetEnvironmentPaths::Fn as NativeGetEnvironmentPaths,
	OSProperties::Fn as NativeOSProperties,
	OSStatistics::Fn as NativeOSStatistics,
	IsFullscreen::Fn as NativeIsFullscreen,
	IsMaximized::Fn as NativeIsMaximized,
	PickFolder::Fn as NativePickFolder,
	ShowMessageBox::Fn as NativeShowMessageBox,
	ShowOpenDialog::Fn as NativeShowOpenDialog,
	ShowSaveDialog::Fn as NativeShowSaveDialog,
	ShowSaveDialogUI::Fn as UserInterfaceShowSaveDialog,
};

/// Dispatches native host dialog and environment commands.
pub async fn dispatch_native_host_dialogs(
	app_handle:&tauri::AppHandle,

	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

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
