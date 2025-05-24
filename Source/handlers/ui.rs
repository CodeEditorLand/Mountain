// ---------------------------------------------------------------------------------------------
// Mountain UI Handlers (handlers/ui.rs)
// --------------------------------------------------------------------------------------------
// Contains basic handlers for UI-related actions proxied from sidecars,
// primarily simple notifications like `showInformationMessage`. More complex UI
// interactions (input boxes, quick picks) would require more elaborate handlers
// and potentially custom UI components in the frontend (Sky).
//
// Responsibilities:
// - Handling `ui_showMessage` requests (or similar) proxied from Cocoon shims.
// - Parsing message content and severity level.
// - Using Tauri's native dialog API (`tauri::api::dialog::message`) to display
//   the message to the user.
// - Returning results (e.g., selected button ID) if the UI interaction supports
//   it (currently returns null for simple messages).
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (or UI effects).
// - Uses Tauri's Dialog API (`tauri::api::dialog`).
// - Needs `Window` context to display the dialog appropriately.
// --------------------------------------------------------------------------------------------

use serde_json::{Value, json};
use tauri::{Runtime, Window};

// Handler for ui_showMessage proxied from Cocoon shim
pub async fn handle_show_message<R:Runtime>(window:Window<R>, params:Value) -> Result<Value, String> {
	let severity = params.get("severity").and_then(|v| v.as_u64()).unwrap_or(2); // Default Info
	let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("");
	println!("[UI Handler] ShowMessage severity={}, message={}", severity, message);

	// Map severity if needed
	let title = match severity {
		1 => "Extension Warning", // Warn
		0 => "Extension Error",   // Error
		_ => "Extension Info",    // Info/default
	};

	// Use Tauri's dialog API
	tauri::api::dialog::message(Some(&window), title, message);

	// The original API can have buttons and returns the selected button.
	// For MVP shim, we ignore buttons and return null/undefined.
	Ok(Value::Null)
}
