// ---------------------------------------------------------------------------------------------
// Mountain UI Handlers (handlers/ui.rs) - BASIC IMPLEMENTATIONS / PLACEHOLDERS
// --------------------------------------------------------------------------------------------
// Contains basic handlers for UI-related actions proxied from sidecars (e.g.,

// Cocoon), primarily for simple notifications like
// `vscode.window.showInformationMessage`.
//
// **CURRENT STATUS: VERY BASIC / PARTIAL IMPLEMENTATION.**
//
// More complex UI interactions (e.g., input boxes, quick picks, dialogs with
// buttons and return values) are intended to be handled by the `UiProvider`
// trait implemented in `environment.rs`. That system uses a request-response
// pattern involving Tauri events to Sky (`sky://ui/...-request`) and callback
// commands from Sky (`sky_resolves_ui_request`).
//
// The handlers in this file, like `handle_show_message_basic`, represent
// either:
// 1. An older, simpler approach to UI interactions.
// 2. A direct RPC handler for very simple, fire-and-forget UI notifications
//    that don't require a response beyond acknowledgement.
//
// If `vscode.window.showInformationMessage` (and similar `showWarningMessage`,

// `showErrorMessage`) are routed to call methods in `rpc.rs` which then call
// `handle_show_message_basic`, this file would serve that purpose. However,

// the more robust `UiProvider` pattern in `environment.rs` is preferred for
// consistency and richer interactions.
//
// Responsibilities (for `handle_show_message_basic`):
// - Handling `ui_showMessage` (or similar) requests proxied from Cocoon shims,

//   if routed here.
// - Parsing message content and severity level from parameters.
// - Using Tauri's native dialog API (`tauri::api::dialog::message`) to display
//   a simple, non-blocking message to the user.
// - Returning a null/void result, as these simple dialogs don't typically
//   return choices.
//
// Key Interactions:
// - Potentially called by `rpc.rs` methods (e.g., `MainThreadMessageHandler`)
//   if requests are routed that way.
// - Uses Tauri's Dialog API (`tauri::api::dialog`).
// - Requires `Window` context to display the dialog appropriately.
// --------------------------------------------------------------------------------------------

// For logging
use log::{debug, info, warn};
// `json!` macro likely unused here, `Value` is key.
use serde_json::{Value, json};
use tauri::{Runtime, Window};

// TODO: Review if this `handle_show_message_basic` is still the primary way
//       `vscode.window.showXMessage` calls are handled, or if they all go
//       through the `UiProvider` effect system in `environment.rs`.
//       If the latter, this handler might be redundant or only for very
// specific,       simple cases not requiring the full effect/callback flow.
/// Handles a basic `ui_showMessage` request, typically proxied from a Cocoon
/// shim for `vscode.window.showInformationMessage` and similar.
///
/// This function displays a simple, non-modal, native OS dialog with the
/// provided message and severity. It does not support buttons or return values
/// from the dialog. For more complex dialogs, the `UiProvider` effect system
/// should be used.
///
/// # Arguments
/// * `window` - The Tauri `Window` context in which to display the dialog.
/// * `params` - A `serde_json::Value` object expected to contain:
///   - `severity`: Optional `u64` (0=Error, 1=Warning, 2=Info - VS Code like).
///     Defaults to Info (2).
///   - `message`: Optional `string` containing the message to display. Defaults
///     to an empty string.
///   - `options`: Optional `object` for message options (e.g., `modal`,
///     `items`). This basic handler currently IGNORES these options.
///
/// # Returns
/// * `Ok(Value::Null)` as this basic dialog does not return a selection.
/// * `Err(String)` is not typically returned by this simple handler unless a
///   future version adds parameter validation that can fail.
pub async fn handle_show_message_basic<R:Runtime>(
	window:Window<R>,

	// Expected: { severity?: number, message?: string, options?: object }
	params:Value,
) -> Result<Value, String> {
	// Default severity to Info (2) if not provided or not a number.
	// VS Code severity mapping: Error=0, Warning=1, Info=2 (can vary, this is one
	// common scheme) Let's align with `Land_Common::ui_effects::MessageSeverity`
	// if this is called directly: Error = 0, Warning = 1, Info = 2
	// Default to Info
	let severity_num = params.get("severity").and_then(|v| v.as_u64()).unwrap_or(2);

	let message_str = params
		.get("message")
		.and_then(|v| v.as_str())
		 // Default to empty string
		.unwrap_or("")
		.to_string();

	// Options might include `modal: boolean` or `items: string[]` for buttons.
	// This basic handler ignores them. The `UiProvider` effect handles them.
	let _options_val = params.get("options");

	debug!(
		"[UI Handler Basic] ShowMessage: severity_num={}, message='{}...', options_present={}",
		severity_num,
		message_str.chars().take(50).collect::<String>(),
		_options_val.is_some()
	);

	if _options_val.is_some() && !_options_val.unwrap().as_object().map_or(true, |o| o.is_empty()) {
		warn!(
			"[UI Handler Basic] Received message options, but this basic handler ignores them (e.g., buttons, \
			 modality). For richer dialogs, use UiProvider effects. Options: {:?}",
			_options_val
		);
	}

	// Map severity number to a title prefix.
	let title_prefix = match severity_num {
		// Error
		0 => "Error",

		// Warning
		1 => "Warning",

		// Info (default for 2 and any other value)
		_ => "Info",
	};

	// TODO: Get application name from AppHandle or config for a more dynamic title.
	let dialog_title = format!("Land Editor - {}", title_prefix);

	// Use Tauri's asynchronous dialog API.
	// `tauri::api::dialog::message` is simple and non-blocking for the main thread
	// if called directly, but here it's in an async fn.
	// It shows a native OS dialog.
	// Note: `tauri::api::dialog` functions are convenient but might block the
	// calling async task if not spawned onto a blocking thread pool, depending on
	// Tauri's internal implementation. For fire-and-forget messages, this is often
	// acceptable. If this were a critical path, `window.dialog().message(...)`
	// might offer more control.
	// Clone for potential async dialog call if needed.
	let window_clone = window.clone();

	// `tauri::api::dialog::message` is synchronous in its current public form.
	// To make it non-blocking for this async handler, it should ideally be
	// spawned if it were a long operation, or if the API offered an async version.
	// However, simple message dialogs are usually quick.

	// Direct call for simplicity, as it's a fire-and-forget notification.
	tauri::api::dialog::message(Some(&window_clone), dialog_title, message_str);

	info!("[UI Handler Basic] Native dialog displayed for severity_num={}.", severity_num);

	// Simple `showInformationMessage` (and equivalents) in VS Code API
	// can have buttons and return the selected button's title (string) or
	// undefined if dismissed.
	// This basic handler doesn't support buttons, so it always effectively returns
	// "undefined" (represented by `Value::Null`).
	Ok(Value::Null)
}

// TODO: Add handlers for other simple UI notifications if needed, e.g.,

//       - `handle_show_status_bar_message`: Would emit a Tauri event to Sky.
//       - `handle_set_progress`: Could also emit events for Sky to display
//         progress.
//       However, these are also good candidates for the `UiProvider` effect
// system       for consistency, even if they don't require a response from Sky.
