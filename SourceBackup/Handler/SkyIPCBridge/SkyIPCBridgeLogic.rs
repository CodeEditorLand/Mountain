// @module SkyIPCBridgeLogic
// @description Contains the logic for bridging generic IPC messages from the
// Sky frontend to the Cocoon sidecar. This acts as a proxy for VS Code's
// legacy `IPCRenderer` communication pattern.

use log::debug;
use serde::Deserialize;
use serde_json::Value;
use tauri::command;

use crate::{handler::error_utils, Vine};

const DEFAULT_SIDECAR_ID:&str = "cocoon-main";
const DEFAULT_TIMEOUT_MS:u64 = 30000;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IpcBridgeArgument {
	pub Channel:String,
	#[serde(default)]
	pub ArgumentList:Vec<Value>,
}

/// A Tauri command that forwards a "send" (fire-and-forget) message from Sky to
/// Cocoon. This effectively shims `IPCRenderer.send`.
#[command(rename_all = "PascalCase")]
pub async fn MountainIpcBridgeSend(argument:IpcBridgeArgument) -> Result<(), String> {
	debug!(
		"[SkyIPCBridge] Forwarding 'send' notification on channel '{}'",
		argument.Channel
	);

	// Construct a unique method name that the Cocoon dispatcher can recognize.
	let Vine_method_name = format!("IPC:send:{}", argument.Channel);
	let Vine_parameters = Value::Array(argument.ArgumentList);

	Vine::client::SendNotification(DEFAULT_SIDECAR_ID.to_string(), Vine_method_name, Vine_parameters)
		.await
		.map_err(|e| error_utils::MapCommonErrorToRpcString(e, "IpcBridgeSend"))
}

/// A Tauri command that forwards an "invoke" (request-response) message from
/// Sky to Cocoon. This effectively shims `IPCRenderer.invoke`.
#[command(rename_all = "PascalCase")]
pub async fn MountainIpcBridgeInvoke(argument:IpcBridgeArgument) -> Result<Value, String> {
	debug!("[SkyIPCBridge] Forwarding 'invoke' request on channel '{}'", argument.Channel);

	let Vine_method_name = format!("IPC:invoke:{}", argument.Channel);
	let Vine_parameters = Value::Array(argument.ArgumentList);

	Vine::client::SendRequest(
		DEFAULT_SIDECAR_ID.to_string(),
		Vine_method_name,
		Vine_parameters,
		DEFAULT_TIMEOUT_MS,
	)
	.await
	.map_err(|e| error_utils::MapCommonErrorToRpcString(e, "IpcBridgeInvoke"))
}
