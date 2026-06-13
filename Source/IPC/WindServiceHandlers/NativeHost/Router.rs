//! NativeHost command router.
//!
//! Routes all `nativeHost:*` commands to their handlers. Atomic handlers
//! live in sibling module files; small stubs are inlined here.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{IPC::WindServiceHandlers::NativeHost, RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Routes nativeHost commands. Returns `Some(result)` for handled commands,
/// `None` otherwise.
pub(crate) async fn route(
	ApplicationHandle:tauri::AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		// ── Dialogs ─────────────────────────────────────────────────────
		"nativeHost:pickFolderAndOpen"
		| "nativeHost:pickFileAndOpen"
		| "nativeHost:pickFileFolderAndOpen"
		| "nativeHost:pickWorkspaceAndOpen" => Some(NativeHost::PickFolder::Fn(ApplicationHandle, Arguments).await),

		"nativeHost:showOpenDialog" => Some(NativeHost::ShowOpenDialog::Fn(ApplicationHandle, Arguments).await),

		"nativeHost:showSaveDialog" => Some(NativeHost::ShowSaveDialog::Fn(ApplicationHandle, Arguments).await),

		"nativeHost:showMessageBox" => Some(NativeHost::ShowMessageBox::Fn(ApplicationHandle, Arguments).await),

		// ── Environment / OS info ───────────────────────────────────────
		"nativeHost:getEnvironmentPaths" => Some(NativeHost::GetEnvironmentPaths::Fn(ApplicationHandle).await),

		"nativeHost:getWebSocketConfig" => Some(NativeHost::GetWebSocketConfig::Fn().await),

		"nativeHost:getOSColorScheme" => {
			dev_log!("nativehost", "nativeHost:getOSColorScheme");

			Some(NativeHost::GetColorScheme::Fn().await)
		},

		"nativeHost:getOSProperties" => {
			dev_log!("nativehost", "nativeHost:getOSProperties");

			Some(NativeHost::OSProperties::Fn().await)
		},

		"nativeHost:getOSStatistics" => {
			dev_log!("nativehost", "nativeHost:getOSStatistics");

			Some(NativeHost::OSStatistics::Fn().await)
		},

		"nativeHost:getOSVirtualMachineHint" => {
			dev_log!("nativehost", "nativeHost:getOSVirtualMachineHint");

			Some(Ok(json!(0)))
		},

		// ── Window state queries ────────────────────────────────────────
		"nativeHost:isWindowAlwaysOnTop" => {
			dev_log!("window", "nativeHost:isWindowAlwaysOnTop");

			Some(Ok(json!(false)))
		},

		"nativeHost:isFullScreen" => {
			dev_log!("window", "nativeHost:isFullScreen");

			Some(NativeHost::IsFullscreen::Fn(ApplicationHandle).await)
		},

		"nativeHost:isMaximized" => {
			dev_log!("window", "nativeHost:isMaximized");

			Some(NativeHost::IsMaximized::Fn(ApplicationHandle).await)
		},

		"nativeHost:getActiveWindowId" => {
			dev_log!("window", "nativeHost:getActiveWindowId");

			Some(Ok(json!(1)))
		},

		"nativeHost:getCursorScreenPoint" => {
			dev_log!("window", "nativeHost:getCursorScreenPoint");

			Some(Ok(json!({ "x": 0, "y": 0 })))
		},

		"nativeHost:getWindows" => Some(NativeHost::GetWindows::Fn(RunTime).await),

		"nativeHost:getWindowCount" => Some(Ok(json!(1))),

		// ── Auxiliary window stubs ──────────────────────────────────────
		"nativeHost:openAgentsWindow" | "nativeHost:openDevToolsWindow" | "nativeHost:openAuxiliaryWindow" => {
			dev_log!("window", "{} (acknowledged, no-op - aux window unsupported)", command);

			Some(Ok(Value::Null))
		},

		// ── Window control (AppHandle-managed) ──────────────────────────
		"nativeHost:focusWindow" => Some(NativeHost::FocusWindow::Fn(&ApplicationHandle, command)),

		"nativeHost:maximizeWindow" => Some(NativeHost::MaximizeWindow::Fn(&ApplicationHandle, command)),

		"nativeHost:unmaximizeWindow" => Some(NativeHost::UnmaximizeWindow::Fn(&ApplicationHandle, command)),

		"nativeHost:minimizeWindow" => Some(NativeHost::MinimizeWindow::Fn(&ApplicationHandle, command)),

		"nativeHost:toggleFullScreen" => Some(NativeHost::ToggleFullScreen::Fn(&ApplicationHandle, command)),

		"nativeHost:onDidChangeMaximizeState" => {
			Some(NativeHost::OnDidChangeMaximizeState::Fn(&ApplicationHandle, &Arguments))
		},

		"nativeHost:closeWindow" => Some(NativeHost::CloseWindow::Fn(&ApplicationHandle, command)),

		"nativeHost:setWindowAlwaysOnTop" => {
			Some(NativeHost::SetWindowAlwaysOnTop::Fn(&ApplicationHandle, command, &Arguments))
		},

		"nativeHost:toggleWindowAlwaysOnTop" => {
			Some(NativeHost::ToggleWindowAlwaysOnTop::Fn(&ApplicationHandle, command))
		},

		"nativeHost:setRepresentedFilename" => {
			Some(NativeHost::SetRepresentedFilename::Fn(&ApplicationHandle, command, &Arguments))
		},

		"nativeHost:setTitle" => Some(NativeHost::SetTitle::Fn(&ApplicationHandle, command, &Arguments)),

		"nativeHost:setDocumentEdited" => Some(NativeHost::SetDocumentEdited::Fn(&ApplicationHandle, &Arguments)),

		"nativeHost:setMinimumSize" => Some(NativeHost::SetMinimumSize::Fn(&ApplicationHandle, &Arguments)),

		"nativeHost:positionWindow" => Some(NativeHost::PositionWindow::Fn(&ApplicationHandle, &Arguments)),

		// ── No-op lifecycle/cosmetic signals ────────────────────────────
		"nativeHost:updateWindowControls"
		| "nativeHost:notifyReady"
		| "nativeHost:saveWindowSplash"
		| "nativeHost:updateTouchBar"
		| "nativeHost:moveWindowTop"
		| "nativeHost:setBackgroundThrottling"
		| "nativeHost:updateWindowAccentColor" => {
			dev_log!("window", "{}", command);

			Some(Ok(Value::Null))
		},

		// ── OS operations ───────────────────────────────────────────────
		"nativeHost:isAdmin" => Some(Ok(json!(false))),

		"nativeHost:isRunningUnderARM64Translation" => Some(NativeHost::IsRunningUnderARM64Translation::Fn().await),

		"nativeHost:hasWSLFeatureInstalled" => Some(NativeHost::HasWSLFeatureInstalled::Fn().await),

		"nativeHost:showItemInFolder" => Some(NativeHost::ShowItemInFolder::Fn(RunTime, Arguments).await),

		"nativeHost:openExternal" => Some(NativeHost::OpenExternal::Fn(RunTime, Arguments).await),

		"nativeHost:moveItemToTrash" => {
			dev_log!("nativehost", "nativeHost:moveItemToTrash");

			Some(NativeHost::MoveItemToTrash::Fn(Arguments).await)
		},

		// ── Clipboard ───────────────────────────────────────────────────
		"nativeHost:readClipboardText" => {
			dev_log!("clipboard", "readClipboardText");

			Some(NativeHost::ClipboardReadText::Fn(Arguments).await)
		},

		"nativeHost:writeClipboardText" => {
			dev_log!("clipboard", "writeClipboardText");

			Some(NativeHost::ClipboardWriteText::Fn(Arguments).await)
		},

		"nativeHost:readClipboardFindText" => {
			dev_log!("clipboard", "readClipboardFindText");

			Some(NativeHost::ClipboardReadFindText::Fn(Arguments).await)
		},

		"nativeHost:writeClipboardFindText" => {
			dev_log!("clipboard", "writeClipboardFindText");

			Some(NativeHost::ClipboardWriteFindText::Fn(Arguments).await)
		},

		"nativeHost:readClipboardBuffer" => {
			dev_log!("clipboard", "readClipboardBuffer");

			Some(NativeHost::ClipboardReadBuffer::Fn(Arguments).await)
		},

		"nativeHost:writeClipboardBuffer" => {
			dev_log!("clipboard", "writeClipboardBuffer");

			Some(NativeHost::ClipboardWriteBuffer::Fn(Arguments).await)
		},

		"nativeHost:hasClipboard" => {
			dev_log!("clipboard", "hasClipboard");

			Some(NativeHost::ClipboardHas::Fn(Arguments).await)
		},

		"nativeHost:readImage" => {
			dev_log!("clipboard", "readImage");

			Some(NativeHost::ClipboardReadImage::Fn(Arguments).await)
		},

		"nativeHost:triggerPaste" => {
			dev_log!("clipboard", "triggerPaste");

			Some(NativeHost::ClipboardTriggerPaste::Fn(Arguments).await)
		},

		// ── Process / Network ───────────────────────────────────────────
		"nativeHost:getProcessId" => Some(Ok(json!(std::process::id()))),

		"nativeHost:killProcess" => Some(NativeHost::KillProcess::Fn(Arguments).await),

		"nativeHost:findFreePort" => Some(NativeHost::FindFreePort::Fn(Arguments).await),

		"nativeHost:isPortFree" => Some(NativeHost::IsPortFree::Fn(Arguments).await),

		"nativeHost:resolveProxy" => Some(NativeHost::ResolveProxy::Fn(Arguments)),

		"nativeHost:lookupAuthorization" => Some(Ok(json!({"username":"","password":""}))),

		"nativeHost:lookupKerberosAuthorization" => Some(Ok(Value::Null)),

		"nativeHost:loadCertificates" => Some(Ok(json!([]))),

		// ── Lifecycle ───────────────────────────────────────────────────
		"nativeHost:relaunch" => Some(NativeHost::Relaunch::Fn(ApplicationHandle, Arguments).await),

		"nativeHost:reload" => Some(NativeHost::Reload::Fn(ApplicationHandle, Arguments).await),

		"nativeHost:quit" => Some(NativeHost::Quit::Fn(ApplicationHandle, Arguments).await),

		"nativeHost:exit" => Some(NativeHost::Exit::Fn(ApplicationHandle, Arguments).await),

		// ── Dev tools ───────────────────────────────────────────────────
		"nativeHost:openDevTools" => Some(NativeHost::OpenDevTools::Fn(ApplicationHandle, Arguments).await),

		"nativeHost:toggleDevTools" => Some(NativeHost::ToggleDevTools::Fn(ApplicationHandle, Arguments).await),

		// ── Power ───────────────────────────────────────────────────────
		"nativeHost:getSystemIdleState" => Some(NativeHost::GetSystemIdleState::Fn()),

		"nativeHost:getSystemIdleTime" => Some(NativeHost::GetSystemIdleTime::Fn()),

		"nativeHost:getCurrentThermalState" => Some(Ok(json!("nominal"))),

		"nativeHost:isOnBatteryPower" => Some(Ok(json!(false))),

		"nativeHost:startPowerSaveBlocker" => Some(NativeHost::StartPowerSaveBlocker::Fn()),

		"nativeHost:stopPowerSaveBlocker" => Some(Ok(json!(false))),

		"nativeHost:isPowerSaveBlockerStarted" => Some(Ok(json!(false))),

		// ── macOS tab stubs ─────────────────────────────────────────────
		"nativeHost:newWindowTab"
		| "nativeHost:showPreviousWindowTab"
		| "nativeHost:showNextWindowTab"
		| "nativeHost:moveWindowTabToNewWindow"
		| "nativeHost:mergeAllWindowTabs"
		| "nativeHost:toggleWindowTabsBar" => Some(Ok(Value::Null)),

		// ── Shell command install ───────────────────────────────────────
		"nativeHost:installShellCommand" => Some(NativeHost::InstallShellCommand::Fn(Arguments).await),

		"nativeHost:uninstallShellCommand" => Some(NativeHost::UninstallShellCommand::Fn(Arguments).await),

		// ── Not a nativeHost command this router handles ────────────────
		_ => None,
	}
}
