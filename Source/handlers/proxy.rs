use serde_json::{Value, json};
use tauri::{AppHandle, Runtime, Window};

// Use the Vine IPC layer
use crate::vine;

pub async fn handle_ext_host_proxy<R:Runtime>(
	_app:AppHandle<R>,
	_window:Window<R>,
	args:Vec<Value>,
) -> Result<Value, String> {
	println!("[Proxy Handler] Proxying call to Cocoon: {:?}", args);

	// TODO: Identify target Cocoon process instance
	// Placeholder
	let target_process_id = 1234;

	// TODO: Robust arg handling - expect specific structure for API call
	let request_payload = args.get(0).cloned().unwrap_or(json!(null));

	// Use Vine to send the request and await response
	match vine::send_to_sidecar(target_process_id, request_payload).await {
		Ok(response) => Ok(response),
		Err(e) => Err(format!("Cocoon IPC Error: {}", e)),
	}
}
