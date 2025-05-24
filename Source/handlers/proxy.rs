// ---------------------------------------------------------------------------------------------
// Mountain Extension Host Proxy Handler (handlers/proxy.rs) - LIKELY
// DEPRECATED/UNUSED
// --------------------------------------------------------------------------------------------
// This file appears to contain a generic proxy handler
// (`handle_ext_host_proxy`) intended to forward arbitrary calls to a Cocoon (or
// other sidecar) process.
//
// **CURRENT STATUS: LIKELY DEPRECATED OR AN EARLY CONCEPT.**
//
// Modern interaction with Cocoon is primarily handled through:
// 1. Specific RPC method handlers defined in `rpc.rs` (mirroring VS Code's
//    `MainThread...Shape` interfaces).
// 2. Direct effect creation in `track.rs` for common operations, which bypasses
//    generic RPC dispatch.
// 3. The `vine.rs` IPC layer for structured request/response/notification
//    patterns.
//
// A generic, untyped proxy like this is generally discouraged due to lack of
// type safety, difficulty in versioning, and potential security risks if not
// carefully managed.
//
// If this was intended for a specific purpose not covered by the above, its
// role needs clarification. Otherwise, it's likely superseded by more robust
// IPC mechanisms.
//
// This file is documented based on its apparent intent but should be reviewed
// for its current relevance and necessity in the Mountain architecture.
// --------------------------------------------------------------------------------------------

// For logging
use log::{debug, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime, Window};

// Use the Vine IPC layer if this were to make actual calls
use crate::vine;

// TODO: Determine if this handler is still used or if it's a remnant of an
// earlier design. If unused, it should be removed. If used, its purpose and
// the structure of `args` need to be clearly defined.

/// **DEPRECATED/UNUSED?** Handles generic proxying of calls to a Cocoon sidecar
/// process.
///
/// This function takes a vector of `serde_json::Value` arguments, assumes the
/// first argument is the payload to send, and attempts to forward it to a
/// hardcoded target process ID via Vine.
///
/// **Warning:** This is a highly generic and potentially unsafe proxy. It lacks
/// specific method dispatch, argument validation, and clear error handling for
/// the proxied call itself beyond IPC errors. Its use is discouraged in favor
/// of specific, well-defined RPC interfaces or effects.
///
/// # Arguments
/// * `_app` - The Tauri `AppHandle` (currently unused).
/// * `_window` - The Tauri `Window` (currently unused).
/// * `args` - A `Vec<Value>` where the first element is expected to be the
///   `request_payload` for the sidecar.
///
/// # Returns
/// * `Ok(Value)` with the response from the sidecar if the IPC call succeeds.
/// * `Err(String)` if `args` is empty, or if the Vine IPC call fails.
pub async fn handle_ext_host_proxy_passthrough<R:Runtime>(
	// Unused
	_app:AppHandle<R>,

	// Unused
	_window:Window<R>,

	args:Vec<Value>,
) -> Result<Value, String> {
	warn!(
		"[Proxy Handler - DEPRECATED?] Received generic proxy call to Cocoon. Args: {:?}. This handler's usage should \
		 be reviewed.",
		args
	);

	// TODO: Identify the target Cocoon process instance dynamically or via config.
	//       Using a hardcoded ID is not robust.
	// Placeholder - THIS IS INVALID AND NEEDS REPLACEMENT if used.
	let target_process_id = 1234;

	// Sidecars should be identified by string IDs like "cocoon-main".

	// TODO: Robust argument handling is critical here.
	//       - What is the expected structure of `args`?
	//       - How is the method for Cocoon determined? Is it part of
	//         `request_payload`?
	//       - What timeout should be used?
	let request_payload = args.get(0).cloned().unwrap_or_else(|| {
		// If no arguments, what should be sent? Defaulting to null.
		warn!("[Proxy Handler] No arguments provided for proxy call. Sending null payload.");

		json!(null)
	});

	debug!(
		"[Proxy Handler] Proxying payload to Cocoon (target process ID placeholder: {}): {:?}",
		target_process_id, request_payload
	);

	// TODO: If this is meant to be a generic RPC passthrough, the `method` name for
	//       Vine is missing. `vine::send_to_sidecar` is a low-level function that
	//       sends a raw VineMessage. Higher-level `vine::send_request_to_sidecar`
	//       should be used, which requires a method name.
	//
	// Example of how it *might* be used if `request_payload` contained
	// method/params: let method_name =
	// request_payload.get("method").and_then(Value::as_str).unwrap_or("
	// unknownMethod"); let params_val =
	// request_payload.get("params").cloned().unwrap_or(json!(null));

	// 30 seconds, example
	// let timeout_ms = 30000;

	// match vine::send_request_to_sidecar(
	// Use string ID
	//     "cocoon-main",

	//     method_name.to_string(),

	//     params_val,

	//     timeout_ms
	// ).await {

	//     Ok(response) => Ok(response),

	//     Err(e) => Err(format!("Cocoon IPC Error (via generic proxy): {}", e)),

	// }

	// The original `vine::send_to_sidecar(target_process_id, request_payload)`
	// is not a valid call as `send_to_sidecar` doesn't exist in the provided
	// `vine.rs`. Assuming it meant to use a function that sends a raw message or a
	// request. For now, returning an error as the original call is unmappable.
	Err(format!(
		"Generic proxy handler (handle_ext_host_proxy_passthrough) is not correctly implemented or is deprecated. \
		 Target Process ID: {}, Payload: {:?}",
		// This ID is problematic.
		target_process_id,
		request_payload
	))
}
