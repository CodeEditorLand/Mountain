// ---------------------------------------------------------------------------------------------
// Mountain Extension Enablement Handlers (handlers/enablement.rs)
// --------------------------------------------------------------------------------------------
// Provides handlers for RPC requests from Cocoon related to querying and
// potentially modifying the enablement state of extensions.
//
// In a full implementation, this would interact with a more complex extension
// management service within Mountain that tracks which extensions are enabled
// globally, per workspace, etc. For the MVP, it provides a simplified response.
//
// Responsibilities:
// - Handling `$getEnablementState` RPC calls from Cocoon's
//   `extensionEnablementService` shim.
// - Returning a mock enablement state.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` or `rpc.rs`.
// - (Future) Would interact with `AppState` or a dedicated extension service to
//   get actual enablement states.
// --------------------------------------------------------------------------------------------

// Potentially use log for debugging
use log::{debug, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};

// Assume Mountain has access to the real IWorkbenchExtensionEnablementService
// state via AppState or a dedicated service in the future.

// Corresponds to vscode.ExtensionEnablementState
// Disabled = 0, EnabledGlobally = 1, EnabledWorkspace = 2, DisabledGlobally =
// -1, DisabledWorkspace = -2
// Using i32 for direct JSON representation.
const EXTENSION_ENABLEMENT_STATE_ENABLED_GLOBALLY:i32 = 1;

// Example
// const EXTENSION_ENABLEMENT_STATE_DISABLED: i32 = 0;

/// Handles the `$getEnablementState` RPC request from Cocoon.
///
/// This method is called by the extension enablement service shim in Cocoon to
/// determine if an extension is enabled.
///
/// # Argument
/// * `_app` - The Tauri `AppHandle` (currently unused).
/// * `params` - A `serde_json::Value` expected to be an object containing an
///   `extensionId` field (which itself is an object like `{value: "pub.name",
///
///   uuid?: "..."}`).
///
/// # Returns
/// * `Ok(Value::Number)` representing the enablement state (e.g., 1 for
///   EnabledGlobally).
/// * `Err(String)` if parameter parsing fails (though currently robustly
///   defaults).
pub async fn handle_get_enablement_state<R:Runtime>(
	// Unused in MVP
	_app:AppHandle<R>,

	params:Value,
) -> Result<Value, String> {
	// Extract extension ID DTO, then the actual string ID from its 'value' field.
	let extension_id_dto = params.get("extensionId");

	let extension_id_str = extension_id_dto
		.and_then(|dto| dto.get("value"))
		.and_then(Value::as_str)
		 // Default if parsing fails
		.unwrap_or("unknown_extension_id");

	debug!(
		"[Enablement Handler] GetState request for extension ID: '{}'. Full DTO: {:?}",
		extension_id_str, extension_id_dto
	);

	// TODO: Query the actual enablement service state in Mountain.
	//       This would involve checking AppState or a dedicated service that
	// tracks:
	//       - Global enablement/disablement settings by user.
	//       - Workspace enablement/disablement settings.
	//       - Whether an extension is built-in (often always enabled).
	//       - Any forced disablement due to errors, etc.

	// For MVP, assume an extension is globally enabled if it was sent to Cocoon
	// in the first place (i.e., it was scanned and included in initData).
	// This is a simplification. A real implementation needs to check stored
	// settings.
	let enabled_state_response = EXTENSION_ENABLEMENT_STATE_ENABLED_GLOBALLY;

	warn!(
		"[Enablement Handler] STUB: Returning mock enablement state ({}) for extension '{}'. Implement full logic.",
		enabled_state_response, extension_id_str
	);

	Ok(json!(enabled_state_response))
}

// TODO: Add handlers for `$setEnablement` RPC calls if the shim requires them.
//       `async fn handle_set_enablement<R: Runtime>(app: AppHandle<R>, params:
// Value) -> Result<Value, String>`       This would involve:
//       1. Parsing extension IDs and the target enablement state
//          (Global/Workspace).
//       2. Updating the persistent configuration (e.g., global settings.json or
//          workspace settings).
//       3. Notifying other parts of the system (and potentially Cocoon) about
//          the change.
//       4. Returning whether a reload/restart is required.
