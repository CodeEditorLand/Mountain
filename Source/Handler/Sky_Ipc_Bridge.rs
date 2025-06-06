// ---------------------------------------------------------------------------------------------
// Mountain Sky IPC Bridge Handlers 
// --------------------------------------------------------------------------------------------
// Implements Tauri command handlers that bridge generic IPC messages (`send`
// and `invoke`) from the Sky frontend to the appropriate sidecar process
// (typically Cocoon, the extension host).
//
// This allows the frontend to communicate with extension APIs or other services
// managed by sidecars using a general-purpose IPC mechanism, similar to VS
// Code's `ipcRenderer.send` and `ipcRenderer.invoke`.
//
// Responsibilities:
// - `mountain_ipc_bridge_send`: Handles fire-and-forget messages from Sky.
//   - Constructs a specific Vine method name (e.g., "ipc:send:channelName").
//   - Forwards the message as a Vine NOTIFICATION to the target sidecar.
// - `mountain_ipc_bridge_invoke`: Handles request-response messages from Sky.
//   - Constructs a specific Vine method name (e.g., "ipc:invoke:channelName").
//   - Forwards the message as a Vine REQUEST to the target sidecar and awaits a
//     response.
// - Uses `crate::vine` for actual IPC communication with sidecars.
// - Provides error formatting for responses to Sky.
//
// Key Interactions:
// - Called by Sky via Tauri `invoke` when using `ipcRenderer.send/invoke`
//   shims.
// - Calls `vine::send_notification_to_sidecar` and
//   `vine::send_request_to_sidecar`.
// - Assumes Cocoon (or other sidecars) are set up to handle Vine methods
//   prefixed with "ipc:send:" and "ipc:invoke:".
// --------------------------------------------------------------------------------------------

use log::{debug, error, trace, warn};
use serde::Deserialize;
use serde_json::Value; // Using Value for args_list for flexibility
use tauri::{AppHandle, Runtime, Wry};

use crate::{handlers::error_utils, vine}; // For sending messages and formatting errors // Wry is the default Tauri runtime

/// Argument for the IPC bridge commands from Sky.
#[derive(Deserialize, Debug)]
pub struct IpcBridgeArgument {
	channel:String, // The IPC channel name (e.g., "vscode:doSomething").
	#[serde(default)] // args_list might be missing if no arguments are sent.
	args_list: Vec<Value>, // Argument for the IPC message.
}

/// Default sidecar ID for extension host operations.
const EXTENSION_HOST_SIDECAR_ID:&str = "cocoon-main";
/// Default timeout for `invoke` calls waiting for a response from the sidecar.
const DEFAULT_INVOKE_TIMEOUT_MS:u64 = 30000; // 30 seconds

/// Tauri command to handle `ipcRenderer.send` style messages from Sky.
/// Forwards the message as a notification to the default extension host
/// sidecar.
#[tauri::command]
pub async fn mountain_ipc_bridge_send(
	_app_handle:AppHandle<Wry>, // Currently unused, but available for future routing logic
	args:IpcBridgeArgument,
) -> Result<(), String> {
	debug!(
		"[SkyIpcBridge Send] Channel='{}', ArgumentCount={}",
		args.channel,
		args.args_list.len()
	);
	trace!("[SkyIpcBridge Send] FullArgument: {:?}", args.args_list);

	if args.channel.starts_with("vscode:electron/") || args.channel.starts_with("vscode:native/") {
		warn!(
			"[SkyIpcBridge Send] Received 'send' for channel '{}' that seems Electron/native specific. Forwarding \
			 as-is, but Cocoon may not handle it.",
			args.channel
		);
	}

	// Construct the Vine method name based on the original channel.
	// Cocoon should have handlers for methods like "ipc:send:channelName".
	let vine_method_name = format!("ipc:send:{}", args.channel);

	// The parameters for the Vine notification will be the original arguments list.
	let vine_params = Value::Array(args.args_list);

	match vine::send_notification_to_sidecar(
		EXTENSION_HOST_SIDECAR_ID.to_string(), // Target sidecar
		vine_method_name.clone(),              // Vine method to call on sidecar
		vine_params,                           // Parameters
	)
	.await
	{
		Ok(_) => {
			debug!(
				"[SkyIpcBridge Send] Successfully forwarded 'send' on channel '{}' (as Vine method '{}') to sidecar \
				 '{}'.",
				args.channel, vine_method_name, EXTENSION_HOST_SIDECAR_ID
			);
			Ok(())
		},
		Err(e) => {
			let err_msg = format!(
				"Failed to forward IPC 'send' on channel '{}' to sidecar '{}': {}",
				args.channel, EXTENSION_HOST_SIDECAR_ID, e
			);
			error!("[SkyIpcBridge Send] {}", err_msg);
			Err(error_utils::rpc_error_string(err_msg, Some("EIPC_FORWARD_SEND_FAIL")))
		},
	}
}

/// Tauri command to handle `ipcRenderer.invoke` style messages from Sky.
/// Forwards the message as a request to the default extension host sidecar and
/// awaits a response.
#[tauri::command]
pub async fn mountain_ipc_bridge_invoke(
	_app_handle:AppHandle<Wry>, // Currently unused, but available for future routing logic
	args:IpcBridgeArgument,
) -> Result<Value, String> {
	// Return type is `Value` because invoke can return anything JSON serializable.
	debug!(
		"[SkyIpcBridge Invoke] Channel='{}', ArgumentCount={}",
		args.channel,
		args.args_list.len()
	);
	trace!("[SkyIpcBridge Invoke] FullArgument: {:?}", args.args_list);

	if args.channel.starts_with("vscode:electron/") || args.channel.starts_with("vscode:native/") {
		warn!(
			"[SkyIpcBridge Invoke] Received 'invoke' for channel '{}' that seems Electron/native specific. Forwarding \
			 as-is, but Cocoon may not handle it or provide a meaningful response.",
			args.channel
		);
	}

	// Construct the Vine method name based on the original channel.
	// Cocoon should have handlers for methods like "ipc:invoke:channelName".
	let vine_method_name = format!("ipc:invoke:{}", args.channel);

	// The parameters for the Vine request will be the original arguments list.
	let vine_request_params = Value::Array(args.args_list);

	match vine::send_request_to_sidecar(
		EXTENSION_HOST_SIDECAR_ID.to_string(), // Target sidecar
		vine_method_name.clone(),              // Vine method to call on sidecar
		vine_request_params,                   // Parameters
		DEFAULT_INVOKE_TIMEOUT_MS,             // Timeout for the request
	)
	.await
	{
		Ok(response_from_cocoon) => {
			debug!(
				"[SkyIpcBridge Invoke] Successfully received response for 'invoke' on channel '{}' (from Vine method \
				 '{}') from sidecar '{}'.",
				args.channel, vine_method_name, EXTENSION_HOST_SIDECAR_ID
			);
			Ok(response_from_cocoon)
		},
		Err(e) => {
			let err_msg = format!(
				"Failed to forward IPC 'invoke' on channel '{}' to sidecar '{}' or get response: {}",
				args.channel, EXTENSION_HOST_SIDECAR_ID, e
			);
			error!("[SkyIpcBridge Invoke] {}", err_msg);
			Err(error_utils::rpc_error_string(err_msg, Some("EIPC_FORWARD_INVOKE_FAIL")))
		},
	}
}
