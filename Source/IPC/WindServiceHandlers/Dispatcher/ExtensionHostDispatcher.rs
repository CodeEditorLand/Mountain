//! ExtensionHost command dispatcher.

use serde_json::Value;

use crate::ExtensionHost::{
	DebugServiceClose::Fn as ExtensionHostDebugClose,
	DebugServiceReload::Fn as ExtensionHostDebugReload,
	StarterCreate::Fn as ExtensionHostStarterCreate,
	StarterGetExitInfo::Fn as ExtensionHostStarterGetExitInfo,
	StarterKill::Fn as ExtensionHostStarterKill,
	StarterStart::Fn as ExtensionHostStarterStart,
	StarterWaitForExit::Fn as ExtensionHostStarterWaitForExit,
};

/// Dispatches extension host commands.
///
/// Handled commands:
/// - `extensionHostStarter:createExtensionHost`
/// - `extensionHostStarter:start`
/// - `extensionHostStarter:kill`
/// - `extensionHostStarter:getExitInfo`
/// - `extensionHostStarter:waitForExit`
/// - `extensionhostdebugservice:reload`
/// - `extensionhostdebugservice:close`
/// - `extensionhostdebugservice:attachSession`
/// - `extensionhostdebugservice:terminateSession`
/// - `cocoon:extensionHostMessage`
pub async fn dispatch_extension_host(
	app_handle:&tauri::AppHandle,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"extensionHostStarter:createExtensionHost" => {
			crate::dev_log!("exthost", "extensionHostStarter:createExtensionHost");

			ExtensionHostStarterCreate(arguments).await
		},

		"extensionHostStarter:start" => {
			crate::dev_log!("exthost", "extensionHostStarter:start");

			ExtensionHostStarterStart(arguments).await
		},

		"extensionHostStarter:kill" => {
			crate::dev_log!("exthost", "extensionHostStarter:kill");

			ExtensionHostStarterKill(arguments).await
		},

		"extensionHostStarter:getExitInfo" => {
			crate::dev_log!("exthost", "extensionHostStarter:getExitInfo");

			ExtensionHostStarterGetExitInfo(arguments).await
		},

		"extensionHostStarter:waitForExit" => {
			crate::dev_log!("exthost", "extensionHostStarter:waitForExit");

			ExtensionHostStarterWaitForExit(arguments).await
		},

		"extensionhostdebugservice:reload" => {
			crate::dev_log!("exthost", "extensionhostdebugservice:reload");

			ExtensionHostDebugReload(app_handle.clone()).await
		},

		"extensionhostdebugservice:close" => {
			crate::dev_log!("exthost", "extensionhostdebugservice:close");

			ExtensionHostDebugClose(app_handle.clone()).await
		},

		// `attachSession` / `terminateSession` require RunTime (for
		// DebugState writes). The primary wired arms live in mod.rs.
		// This dispatcher path is reached only from legacy callers that
		// supply an AppHandle but not a RunTime; acknowledge and no-op.
		"extensionhostdebugservice:attachSession" | "extensionhostdebugservice:terminateSession" => {
			crate::dev_log!("exthost", "{}", command);

			Ok(Value::Null)
		},

		"cocoon:extensionHostMessage" => {
			crate::dev_log!("exthost", "cocoon:extensionHostMessage");

			crate::Cocoon::ExtensionHostMessage::Fn(app_handle.clone(), arguments).await
		},

		_ => Err(format!("Unknown extension host command: {}", command)),
	}
}
