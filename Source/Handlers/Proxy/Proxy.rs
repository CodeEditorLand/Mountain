// File: Handlers/Proxy/Proxy.rs
// Defines a deprecated generic proxy handler for passthrough calls to the
// sidecar. This approach is being replaced by specific, type-safe RPC methods
// and effects.

#![allow(non_snake_case, non_camel_case_types)]

use log::{debug, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime, Window};

/// Handles generic proxy calls from the frontend intended for the Cocoon
/// extension host. This function is deprecated and should not be used in new
/// implementations. It logs a warning and returns an error indicating its
/// deprecated status.
pub async fn HandleExtensionHostProxyPassthrough<R:Runtime>(
	_ApplicationHandle:AppHandle<R>,
	_Window:Window<R>,
	Arguments:Vec<Value>,
) -> Result<Value, String> {
	warn!(
		"[ProxyHandler Deprecated] Generic proxy call intercepted. This handler should be replaced. Arguments: {:?}",
		Arguments
	);

	let TargetProcessIdentifier = 1234; // Placeholder ID

	let RequestPayload = Arguments.get(0).cloned().unwrap_or_else(|| {
		warn!("[ProxyHandler Deprecated] No payload provided for proxy call. Using null.");
		json!(null)
	});

	debug!(
		"[ProxyHandler Deprecated] Proxying payload to Cocoon (placeholder target ID: {}): {:?}",
		TargetProcessIdentifier, RequestPayload
	);

	Err(format!(
		"Generic proxy handler (HandleExtensionHostProxyPassthrough) is deprecated. Target Process ID: {}, Payload: \
		 {:?}",
		TargetProcessIdentifier, RequestPayload
	))
}
