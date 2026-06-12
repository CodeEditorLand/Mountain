//! Extension host commands router.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::{
		Cocoon::ExtensionHostMessage::Fn as CocoonExtensionHostMessage,
		ExtensionHost::{
			DebugServiceClose::Fn as ExtensionHostDebugClose,
			DebugServiceReload::Fn as ExtensionHostDebugReload,
			StarterCreate::Fn as ExtensionHostStarterCreate,
			StarterGetExitInfo::Fn as ExtensionHostStarterGetExitInfo,
			StarterKill::Fn as ExtensionHostStarterKill,
			StarterStart::Fn as ExtensionHostStarterStart,
			StarterWaitForExit::Fn as ExtensionHostStarterWaitForExit,
		},
		Utilities::JsonValueHelpers::{arg_string, arg_string_or, arg_u64},
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes extension host commands. Returns Some(result) for handled commands,
/// None otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	ApplicationHandle:tauri::AppHandle,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		// Extension host starter
		"extensionHostStarter:createExtensionHost" => {
			dev_log!("exthost", "extensionHostStarter:createExtensionHost");

			Some(ExtensionHostStarterCreate(Arguments).await)
		},

		"extensionHostStarter:start" => {
			dev_log!("exthost", "extensionHostStarter:start");

			Some(ExtensionHostStarterStart(Arguments).await)
		},

		"extensionHostStarter:kill" => {
			dev_log!("exthost", "extensionHostStarter:kill");

			Some(ExtensionHostStarterKill(Arguments).await)
		},

		"extensionHostStarter:getExitInfo" => {
			dev_log!("exthost", "extensionHostStarter:getExitInfo");

			Some(ExtensionHostStarterGetExitInfo(Arguments).await)
		},

		"extensionHostStarter:waitForExit" => {
			dev_log!("exthost", "extensionHostStarter:waitForExit");

			Some(ExtensionHostStarterWaitForExit(Arguments).await)
		},

		// Extension host message relay
		"cocoon:extensionHostMessage" => {
			dev_log!("exthost", "cocoon:extensionHostMessage");

			Some(CocoonExtensionHostMessage(ApplicationHandle.clone(), Arguments).await)
		},

		// Extension host debug service
		"extensionhostdebugservice:reload" => {
			dev_log!("exthost", "extensionhostdebugservice:reload");

			Some(ExtensionHostDebugReload(ApplicationHandle.clone()).await)
		},

		"extensionhostdebugservice:close" => {
			dev_log!("exthost", "extensionhostdebugservice:close");

			Some(ExtensionHostDebugClose(ApplicationHandle.clone()).await)
		},

		"extensionhostdebugservice:attachSession" => {
			let SessionId = arg_string(&Arguments, 0);

			let Port = arg_u64(&Arguments, 1);

			let SubId = arg_string_or(&Arguments, 2, "cocoon-main");

			dev_log!(
				"exthost",
				"extensionhostdebugservice:attachSession id={} port={} sub={}",
				SessionId,
				Port,
				SubId
			);

			if !SessionId.is_empty() {
				let AlreadyRegistered = RunTime
					.Environment
					.ApplicationState
					.Feature
					.Debug
					.GetDebugSession(&SessionId)
					.is_some();

				if !AlreadyRegistered {
					let _ = RunTime.Environment.ApplicationState.Feature.Debug.RegisterDebugSession(
						crate::ApplicationState::State::FeatureState::Debug::DebugState::DebugSessionEntry {
							SessionId:SessionId.clone(),
							DebugType:"unknown".to_string(),
							SideCarIdentifier:SubId,
							StdinSender:None,
							ChildPid:None,
						},
					);
				}
			}

			Some(Ok(Value::Null))
		},

		"extensionhostdebugservice:terminateSession" => {
			let SessionId = arg_string(&Arguments, 0);

			dev_log!("exthost", "extensionhostdebugservice:terminateSession id={}", SessionId);

			if !SessionId.is_empty() {
				RunTime
					.Environment
					.ApplicationState
					.Feature
					.Debug
					.UnregisterDebugSession(&SessionId);
			}

			Some(Ok(Value::Null))
		},

		_ => None,
	}
}
