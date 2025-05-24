// ---------------------------------------------------------------------------------------------
// Mountain Diagnostics Handlers (handlers/diagnostics.rs)
// --------------------------------------------------------------------------------------------
// Manages diagnostic information (problems/markers) reported by extensions
// running in sidecars (like Cocoon). Handles RPC calls from Cocoon's
// diagnostics shim, updates AppState, and emits events to notify the frontend
// (Sky).
//
// Responsibilities:
// - Handling `$changeMany` RPC calls.
// - Handling `$clear` RPC calls.
// - Storing diagnostics state in `AppState`.
// - Emitting Tauri events (`diagnostics_changed`) to notify Sky.
// - Handling `$getDiagnostics` RPC calls.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (or effects if these become
//   effects).
// - Interacts with `AppState` to read/write `diagnostics_map`.
// - Uses `serde_json` to deserialize `MarkerData`.
// - Emits Tauri events via `AppHandle::emit_all`.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// StdMutex used if AppState field is direct
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

// Use shared error utilities
use crate::{app_state::AppState, handlers::error_utils};

// --- Helper Functions ---

/// Helper to map Mutex lock poisoning errors for diagnostics state.
fn map_diag_lock_error_to_str<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> String {
	let msg = format!("Failed to acquire lock on diagnostics state: {}", e);

	// Keep specific error log
	error!("[Diag Handler LockErr] {}", msg);

	error_utils::rpc_error_string(msg, Some("ELOCKED"))
}

/// Helper to get a consistent string key from UriComponents Value received via
/// JSON RPC. Primarily uses the 'external' property.
fn get_uri_key_from_components(uri_components:&Value) -> Option<String> {
	if let Some(ext) = uri_components.get("external").and_then(Value::as_str) {
		return Some(ext.to_string());
	}

	// Fallback logic from the first snippet if 'external' isn't always guaranteed.
	// However, VS Code's UriComponents DTO usually includes 'external'.
	// For robustness, let's keep a minimal fallback if the primary isn't there.
	warn!(
		"[Diag Handler] URI components missing 'external' field, attempting fallback: {:?}",
		uri_components
	);

	let scheme = uri_components.get("scheme").and_then(Value::as_str)?;

	let path = uri_components.get("path").and_then(Value::as_str)?;

	let authority = uri_components.get("authority").and_then(Value::as_str).unwrap_or("");

	Some(format!("{}://{}{}", scheme, authority, path))
}

// --- Data Structures ---

// Structure matching vs/platform/markers/common/markers.ts:IMarkerData
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarkerData {
	// Can be string or { value: string, target: UriComponents }
	pub code:Option<Value>,

	// Error=8, Warn=4, Info=2, Hint=1
	pub severity:u32,

	pub message:String,

	pub source:Option<String>,

	#[serde(rename = "startLineNumber")]
	pub start_line_number:u32,

	#[serde(rename = "startColumn")]
	pub start_column:u32,

	#[serde(rename = "endLineNumber")]
	pub end_line_number:u32,

	#[serde(rename = "endColumn")]
	pub end_column:u32,

	#[serde(rename = "modelVersionId")]
	pub model_version_id:Option<u64>,

	#[serde(rename = "relatedInformation")]
	pub related_information:Option<Vec<RelatedInformation>>,

	// Unnecessary=1, Deprecated=2
	pub tags:Option<Vec<u32>>,
}

// Structure matching vs/platform/markers/common/markers.ts:IRelatedInformation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelatedInformation {
	// UriComponents JSON Value
	pub resource:Value,

	pub message:String,

	#[serde(rename = "startLineNumber")]
	pub start_line_number:u32,

	#[serde(rename = "startColumn")]
	pub start_column:u32,

	#[serde(rename = "endLineNumber")]
	pub end_line_number:u32,

	#[serde(rename = "endColumn")]
	pub end_column:u32,
}

// --- RPC Handlers ---

/// Handles the `$changeMany` RPC call from a diagnostics provider.
/// Updates diagnostics state for multiple URIs for a given owner.
/// Args: `[owner: string, entries: [uriComponents: Value, markers: MarkerData[]
/// | null][]]`
pub async fn handle_change_many<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let owner = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("diagnostics_changeMany", "owner", "string", Some(0)))?
		.to_string();

	let entries_val = args
		.get(1)
		.ok_or_else(|| error_utils::rpc_param_error_string("diagnostics_changeMany", "entries", "array", Some(1)))?;

	let entries:Vec<(Value, Option<Vec<MarkerData>>)> = serde_json::from_value(entries_val.clone()).map_err(|e| {
		error_utils::rpc_error_string(format!("Failed to parse 'entries' argument: {}", e), Some("EBADMSG"))
	})?;

	// Keep: Log owner and number of entries summary
	info!("[Diag Handler] changeMany owner='{}', {} entries", owner, entries.len());

	let app_state = app.state::<AppState>();

	let mut changed_uris:Vec<String> = Vec::new();

	{
		// Scope for the mutex lock
		let mut all_owner_diags = app_state.diagnostics_map.lock().map_err(map_diag_lock_error_to_str)?;

		let resource_map = all_owner_diags.entry(owner.clone()).or_default();

		for (uri_components_val, markers_opt) in entries {
			let uri_str = match get_uri_key_from_components(&uri_components_val) {
				Some(s) => s,

				None => {
					warn!(
						"[Diag Handler] changeMany: Skipping entry for owner '{}' with invalid URI components: {:?}",
						owner, uri_components_val
					);

					continue;
				},
			};

			changed_uris.push(uri_str.clone());

			match markers_opt {
				Some(markers) if !markers.is_empty() => {
					// Log detailed marker info at trace level if needed
					trace!(
						"[Diag Handler] Setting {} markers for owner '{}', URI '{}'",
						markers.len(),
						owner,
						uri_str
					);

					resource_map.insert(uri_str, markers);
				},

				_ => {
					// Clear markers (received null, undefined, or empty array)
					trace!("[Diag Handler] Clearing markers for owner '{}', URI '{}'", owner, uri_str);

					resource_map.remove(&uri_str);
				},
			}
		}

		// Clean up owner entry if they have no diagnostics left
		if resource_map.is_empty() {
			info!(
				"[Diag Handler] Owner '{}' has no more diagnostics, removing owner entry.",
				owner
			);

			all_owner_diags.remove(&owner);
		}

		// Lock released here
	}

	if !changed_uris.is_empty() {
		let event_payload = json!({ "owner": owner, "uris": changed_uris });

		// Keep: Log the event being emitted
		debug!("[Diag Handler] Emitting diagnostics_changed event: {:?}", event_payload);

		if let Err(e) = app.emit_all("diagnostics_changed", event_payload) {
			error!("[Diag Handler] Failed to emit diagnostics_changed event: {}", e);
		}
	}

	Ok(Value::Null)
}

/// Handles the `$clear` RPC call from a diagnostics provider.
/// Removes all diagnostics associated with the specified owner.
/// Args: `[owner: string]`
pub async fn handle_clear<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let owner = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("diagnostics_clear", "owner", "string", Some(0)))?
		.to_string();

	// Keep: Clearing all is significant
	info!("[Diag Handler] clear owner='{}'", owner);

	let app_state = app.state::<AppState>();

	let mut cleared_uris:Vec<String> = Vec::new();

	let owner_existed:bool;

	{
		// Scope lock
		let mut all_owner_diags = app_state.diagnostics_map.lock().map_err(map_diag_lock_error_to_str)?;

		if let Some(resource_map) = all_owner_diags.get(&owner) {
			// Get URIs before removal
			cleared_uris = resource_map.keys().cloned().collect();
		}

		// Attempt to remove
		owner_existed = all_owner_diags.remove(&owner).is_some();

		// Lock released here
	}

	if owner_existed {
		// Keep: Confirmation log
		info!("[Diag Handler] Cleared all diagnostics for owner '{}'.", owner);

		if !cleared_uris.is_empty() {
			let event_payload = json!({ "owner": owner, "uris": cleared_uris });

			// Keep: Log the event being emitted
			debug!("[Diag Handler] Emitting diagnostics_changed event (clear): {:?}", event_payload);

			if let Err(e) = app.emit_all("diagnostics_changed", event_payload) {
				error!("[Diag Handler] Failed to emit diagnostics_changed event after clear: {}", e);
			}
		}
	} else {
		// Keep: Warning log
		warn!("[Diag Handler] Owner '{}' not found for clearing.", owner);
	}

	Ok(Value::Null)
}

/// Handles the `$getDiagnostics` RPC call. Aggregates diagnostics, optionally
/// filtered. Args: `[resource?: UriComponents]`
pub async fn handle_get_diagnostics<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let resource_filter_val = args.get(0);

	// Keep trace for debugging filter issues
	trace!("[Diag Handler] getDiagnostics filter='{:?}'", resource_filter_val);

	let app_state = app.state::<AppState>();

	let owner_diags = app_state.diagnostics_map.lock().map_err(map_diag_lock_error_to_str)?;

	let target_uri_str_opt:Option<String> = resource_filter_val
		 // Treat null as no filter
		.filter(|v| !v.is_null())
		.and_then(get_uri_key_from_components);

	let mut aggregated_map:HashMap<String, Vec<MarkerData>> = HashMap::new();

	for (_owner, resource_map) in owner_diags.iter() {
		for (uri_str, markers) in resource_map.iter() {
			if let Some(target_uri) = &target_uri_str_opt {
				if uri_str != target_uri {
					// Apply filter
					continue;
				}
			}

			aggregated_map
				.entry(uri_str.clone())
				.or_default()
				.extend(markers.iter().cloned());
		}
	}

	// Release lock
	drop(owner_diags);

	// Convert aggregated map to expected result format: `[UriComponents,

	// MarkerData[]][]` Ensure UriComponents includes $mid as per VS Code DTOs for
	// revival.
	let result_list:Vec<(Value, Vec<MarkerData>)> = aggregated_map
		.into_iter()
		.map(|(uri_str, markers)| (json!({ "external": uri_str, "$mid": 1 }), markers))
		.collect();

	serde_json::to_value(result_list).map_err(|e| {
		error!("Failed to serialize getDiagnostics result: {}", e);

		error_utils::rpc_error_string(format!("Failed to serialize diagnostics result: {}", e), Some("ESERIALIZE"))
	})
}
