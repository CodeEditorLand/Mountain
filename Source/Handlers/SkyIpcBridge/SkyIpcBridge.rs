// File: Handlers/SkyIpcBridge/SkyIpcBridge.rs
// Defines command handlers that act as a bridge for IPC messages from the Sky
// (frontend), forwarding them to the Cocoon sidecar via the Vine gRPC system.
// This is likely a transitional or deprecated pattern.

#![allow(non_snake_case, non_camel_case_types)]

use log::{debug, trace, warn};
use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Runtime, Wry};

use crate::{Handlers::ErrorUtils, Vine}; // The gRPC communication module

#[derive(Deserialize, Debug)]
pub struct IpcBridgeArgument {
	#[serde(alias = "channel")]
	pub Channel:String,
	#[serde(default, alias = "argsList")]
	pub ArgumentList:Vec<Value>,
}

const EXTENSION_HOST_SIDECAR_IDENTIFIER:&str = "cocoon-main";
const DEFAULT_INVOKE_TIMEOUT_MILLISECONDS:u64 = 30000;

/// Forwards a "send" (fire-and-forget) IPC message from Sky to Cocoon.
#[tauri::command]
pub async fn MountainIpcBridgeSend(
	_ApplicationHandle:AppHandle<Wry>,
	Argument:IpcBridgeArgument,
) -> Result<(), String> {
	debug!(
		"[SkyIpcBridge Send] Channel='{}', ArgumentCount={}",
		Argument.Channel,
		Argument.ArgumentList.len()
	);
	trace!("[SkyIpcBridge Send] FullArgument: {:?}", Argument.ArgumentList);

	// Construct the method name expected by Cocoon's gRPC dispatcher.
	let VineMethodName = format!("ipc:send:{}", Argument.Channel);
	let VineParameters = Value::Array(Argument.ArgumentList);

	match Vine::SendNotification(
		EXTENSION_HOST_SIDECAR_IDENTIFIER.to_string(),
		VineMethodName.clone(),
		VineParameters,
	)
	.await
	{
		Ok(_) => {
			debug!(
				"[SkyIpcBridge Send] Forwarded 'send' on channel '{}' (as Vine method '{}') to sidecar '{}'.",
				Argument.Channel, VineMethodName, EXTENSION_HOST_SIDECAR_IDENTIFIER
			);
			Ok(())
		},
		Err(Error) => {
			let ErrorMessage = format!(
				"Failed to forward IPC 'send' on channel '{}' to sidecar '{}': {}",
				Argument.Channel, EXTENSION_HOST_SIDECAR_IDENTIFIER, Error
			);
			Err(ErrorUtils::RpcErrorString(ErrorMessage, Some("EIPC_FORWARD_SEND_FAIL")))
		},
	}
}

/// Forwards an "invoke" (request/response) IPC message from Sky to Cocoon.
#[tauri::command]
pub async fn MountainIpcBridgeInvoke(
	_ApplicationHandle:AppHandle<Wry>,
	Argument:IpcBridgeArgument,
) -> Result<Value, String> {
	debug!(
		"[SkyIpcBridge Invoke] Channel='{}', ArgumentCount={}",
		Argument.Channel,
		Argument.ArgumentList.len()
	);
	trace!("[SkyIpcBridge Invoke] FullArgument: {:?}", Argument.ArgumentList);

	let VineMethodName = format!("ipc:invoke:{}", Argument.Channel);
	let VineRequestParameters = Value::Array(Argument.ArgumentList);

	match Vine::SendRequest(
		EXTENSION_HOST_SIDECAR_IDENTIFIER.to_string(),
		VineMethodName.clone(),
		VineRequestParameters,
		DEFAULT_INVOKE_TIMEOUT_MILLISECONDS,
	)
	.await
	{
		Ok(ResponseFromCocoon) => {
			debug!(
				"[SkyIpcBridge Invoke] Received response for 'invoke' on channel '{}' from sidecar '{}'.",
				Argument.Channel, EXTENSION_HOST_SIDECAR_IDENTIFIER
			);
			Ok(ResponseFromCocoon)
		},
		Err(Error) => {
			let ErrorMessage = format!(
				"Failed to forward IPC 'invoke' on channel '{}' to sidecar '{}' or get response: {}",
				Argument.Channel, EXTENSION_HOST_SIDECAR_IDENTIFIER, Error
			);
			Err(ErrorUtils::RpcErrorString(ErrorMessage, Some("EIPC_FORWARD_INVOKE_FAIL")))
		},
	}
}
