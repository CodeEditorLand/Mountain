use log::debug;
use serde::Deserialize;
use serde_json::Value;
use tauri::{ApplicationHandle, Wry, command};

// @module SkyIpcBridgeLogic
// @description Contains the logic for bridging generic IPC messages from the
// Sky frontend to the Cocoon sidecar. This acts as a proxy for VS Code's
// legacy `ipcRenderer` communication pattern.
use crate::{Handler::error_utils, vine};

const DEFAULT_SIDECAR_ID:&str = "cocoon-main";
const DEFAULT_TIMEOUT_MS:u64 = 30000;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IpcBridgeArgument {
	pub Channel:String,
	#[serde(default)]
	pub ArgumentList:Vec<Value>,
}

// A Tauri command that forwards a "send" (fire-and-forget) message from Sky to
// Cocoon. This effectively shims `ipcRenderer.send`.
#[command(rename_all = "PascalCase")]
pub async fn MountainIpcBridgeSend(Argument:IpcBridgeArgument) -> Result<(), String> {
	debug!(
		"[SkyIpcBridge] Forwarding 'send' notification on channel '{}'",
		Argument.Channel
	);

	// Construct a unique method name that the Cocoon dispatcher can recognize.
	let VineMethodName = format!("ipc:send:{}", Argument.Channel);
	let VineParameters = Value::Array(Argument.ArgumentList);

	vine::client::SendNotification(DEFAULT_SIDECAR_ID.to_string(), VineMethodName, VineParameters)
		.await
		.map_err(|e| error_utils::MapCommonErrorToRpcString(e, "IpcBridgeSend"))
}

// A Tauri command that forwards an "invoke" (request-response) message from
// Sky to Cocoon. This effectively shims `ipcRenderer.invoke`.
#[command(rename_all = "PascalCase")]
pub async fn MountainIpcBridgeInvoke(Argument:IpcBridgeArgument) -> Result<Value, String> {
	debug!("[SkyIpcBridge] Forwarding 'invoke' request on channel '{}'", Argument.Channel);

	let VineMethodName = format!("ipc:invoke:{}", Argument.Channel);
	let VineParameters = Value::Array(Argument.ArgumentList);

	vine::client::SendRequest(
		DEFAULT_SIDECAR_ID.to_string(),
		VineMethodName,
		VineParameters,
		DEFAULT_TIMEOUT_MS,
	)
	.await
	.map_err(|e| error_utils::MapCommonErrorToRpcString(e, "IpcBridgeInvoke"))
}
