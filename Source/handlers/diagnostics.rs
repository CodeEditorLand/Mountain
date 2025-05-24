// ---------------------------------------------------------------------------------------------
// Mountain Diagnostics Handlers (handlers/diagnostics.rs)
// --------------------------------------------------------------------------------------------
// Manages diagnostic information (problems/markers) reported by extensions
// running in sidecars (like Cocoon). Handles RPC calls from Cocoon's
// diagnostics shim, updates AppState, and emits events to notify the frontend
// (Sky).
//
// Responsibilities:
// - Handling `$changeMany` RPC calls to update or set diagnostics for multiple
//   resources from a specific owner.
// - Handling `$clear` RPC calls to remove all diagnostics for a specific owner.
// - Storing diagnostics state in `AppState.diagnostics_map` (Map<owner,

//   Map<uri_string, Vec<MarkerData>>>).
// - Emitting Tauri events (`diagnostics_changed`) to notify Sky when
//   diagnostics are updated, so the UI can refresh.
// - Handling `$getDiagnostics` RPC calls to retrieve aggregated diagnostics,

//   optionally filtered by resource URI.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (or effects if these become
//   effects).
// - Interacts with `AppState` to read/write `diagnostics_map`.
// - Uses `serde_json` to deserialize `MarkerData` and `RelatedInformation`
//   DTOs.
// - Emits Tauri events via `AppHandle::emit_all`.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// StdMutex used for AppState.diagnostics_map
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

// Use shared error utilities
use crate::{app_state::AppState, handlers::error_utils};

// --- Helper Functions ---

/// Formats a `PoisonError` resulting from a failed Mutex lock on the
/// diagnostics state into a standardized RPC error string.
///
/// # Arguments
/// * `e` - The `PoisonError` encountered.
///
/// # Returns
/// A `String` containing a JSON-formatted RPC error.
fn format_diagnostics_lock_error_for_rpc<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> String {
	let msg = format!("Failed to acquire lock on diagnostics state: {}", e);

	// Keep specific error log for internal diagnostics
	error!("[Diag Handler LockErr] {}", msg);

	// Specific lock error code
	error_utils::rpc_error_string(msg, Some("ELOCKED_DIAG"))
}

/// Helper to get a consistent string key (typically the `external` URI string)
/// from a `serde_json::Value` representing `UriComponents` DTO.
///
/// This is used to key diagnostics by resource URI in the `AppState`.
///
/// # Arguments
/// * `uri_components` - A `serde_json::Value` expected to be an object with at
///   least an `external` field, or fallback to `scheme`, `authority`, `path`.
///
/// # Returns
/// * `Some(String)` with the URI key if successful.
/// * `None` if essential URI components are missing or not strings.
fn get_uri_key_from_uri_components_dto(uri_components:&Value) -> Option<String> {
	// Primary: Use the 'external' field which is the full URI string.
	if let Some(ext_uri_str) = uri_components.get("external").and_then(Value::as_str) {
		return Some(ext_uri_str.to_string());
	}

	// Fallback: Try to construct from scheme, authority, path if 'external' is
	// missing. This is less common for VS Code DTOs but provides robustness.
	warn!(
		"[Diag Helper] URI components DTO missing 'external' field, attempting fallback construction. DTO: {:?}",
		uri_components
	);

	let scheme = uri_components.get("scheme").and_then(Value::as_str)?;

	let path = uri_components.get("path").and_then(Value::as_str)?;

	// Authority can be empty for 'file' URIs, default to empty string if missing.
	let authority = uri_components.get("authority").and_then(Value::as_str).unwrap_or("");

	Some(format!("{}://{}{}", scheme, authority, path))
}

// --- Data Structures (DTOs matching VS Code's `markers.ts`) ---

/// Represents a diagnostic marker (problem, warning, info, hint).
/// Matches `vscode.IMarkerData` (src/vs/platform/markers/common/markers.ts).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarkerData {
	/// The owner of the marker (e.g., "typescript", "eslint"). This field seems
	/// to be part of the `$changeMany` call's top-level args, not per marker
	/// here. The `source` field below is more common per marker.
	// This is usually at a higher level in $changeMany
	// owner: Option<String>,

	/// A code associated with the marker, which can be a string or an object
	/// `{ value: string, target: UriComponents }` (e.g., for a link to
	/// documentation).
	pub code:Option<Value>,

	/// Severity of the marker (Error=8, Warning=4, Info=2, Hint=1).
	/// Matches `vscode.MarkerSeverity`.
	pub severity:u32,

	pub message:String,

	/// The source of the marker (e.g., "tslint", "eslint").
	pub source:Option<String>,

	#[serde(rename = "startLineNumber")]
	// 1-based
	pub start_line_number: u32,

	#[serde(rename = "startColumn")]
	// 1-based
	pub start_column: u32,

	#[serde(rename = "endLineNumber")]
	// 1-based
	pub end_line_number: u32,

	#[serde(rename = "endColumn")]
	// 1-based
	pub end_column: u32,

	/// Optional version ID of the document model this marker applies to.
	#[serde(rename = "modelVersionId")]
	pub model_version_id:Option<u64>,

	/// Optional related information, like quick fixes or secondary locations.
	#[serde(rename = "relatedInformation")]
	pub related_information:Option<Vec<RelatedInformation>>,

	/// Optional tags indicating special properties (e.g., Unnecessary=1,
	///
	///
	///
	/// Deprecated=2). Matches `vscode.MarkerTag`.
	pub tags:Option<Vec<u32>>,
}

/// Represents related information for a `MarkerData`.
/// Matches `vscode.IRelatedInformation`
/// (src/vs/platform/markers/common/markers.ts).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelatedInformation {
	/// The resource URI for this related information (as `UriComponents` JSON
	/// Value).
	pub resource:Value,

	pub message:String,

	#[serde(rename = "startLineNumber")]
	// 1-based
	pub start_line_number: u32,

	#[serde(rename = "startColumn")]
	// 1-based
	pub start_column: u32,

	#[serde(rename = "endLineNumber")]
	// 1-based
	pub end_line_number: u32,

	#[serde(rename = "endColumn")]
	// 1-based
	pub end_column: u32,
}

// --- RPC Handlers ---

/// Handles the `$changeMany` RPC call from a diagnostics provider in Cocoon.
///
/// Updates or sets diagnostics for multiple resource URIs under a specific
/// owner (e.g., "typescript"). If `markers` for an entry is `null` or an empty
/// array, diagnostics for that URI and owner are cleared.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[owner: string, entries:
///   [uriComponents: Value, markers: MarkerData[] | null][] ]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` with a JSON-RPC error if parsing or state update fails.
pub async fn handle_change_many<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let owner = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("diagnostics_changeMany", "owner", "string", Some(0)))?
		.to_string();

	let entries_val = args
		.get(1)
		.ok_or_else(|| error_utils::rpc_param_error_string("diagnostics_changeMany", "entries", "array", Some(1)))?;

	// Deserialize entries. Each entry is a tuple: (UriComponents DTO,

	// Option<Vec<MarkerData DTO>>)
	let entries_deserialized:Vec<(Value, Option<Vec<MarkerData>>)> = serde_json::from_value(entries_val.clone())
		.map_err(|e| {
			error_utils::rpc_error_string(
				format!("Failed to parse 'entries' argument for $changeMany: {}", e),
				Some("EBADMSG_DIAG_ENTRIES"),
			)
		})?;

	// Keep: Log owner and number of entries summary
	info!(
		"[Diag Handler] $changeMany: owner='{}', processing {} resource entries.",
		owner,
		entries_deserialized.len()
	);

	let app_state = app.state::<AppState>();

	// Store string keys of affected URIs
	let mut changed_uri_keys:Vec<String> = Vec::new();

	{
		// Scope for the AppState.diagnostics_map Mutex lock
		let mut all_diagnostics_map_guard = app_state
			.diagnostics_map
			.lock()
			.map_err(format_diagnostics_lock_error_for_rpc)?;

		// Get or create the map for the specific owner
		let owner_specific_diagnostics_map = all_diagnostics_map_guard.entry(owner.clone()).or_default();

		for (uri_components_dto, markers_opt) in entries_deserialized {
			let uri_key_str = match get_uri_key_from_uri_components_dto(&uri_components_dto) {
				Some(s) => s,

				None => {
					warn!(
						"[Diag Handler $changeMany] Skipping entry for owner '{}' due to invalid URI components: {:?}",
						owner, uri_components_dto
					);

					// Skip this entry if URI is unparsable
					continue;
				},
			};

			// Add to list of URIs whose diagnostics changed for this owner.
			if !changed_uri_keys.contains(&uri_key_str) {
				changed_uri_keys.push(uri_key_str.clone());
			}

			match markers_opt {
				Some(markers) if !markers.is_empty() => {
					trace!(
						"[Diag Handler $changeMany] Setting {} markers for owner '{}', URI '{}'",
						markers.len(),
						owner,
						uri_key_str
					);

					owner_specific_diagnostics_map.insert(uri_key_str, markers);
				},

				_ => {
					// Clear markers for this URI if markers_opt is None, or Some(empty_vec).
					trace!(
						"[Diag Handler $changeMany] Clearing markers for owner '{}', URI '{}'",
						owner, uri_key_str
					);

					owner_specific_diagnostics_map.remove(&uri_key_str);
				},
			}
		}

		// If, after updates, an owner has no diagnostics left for any resource,

		// remove the owner's entry from the main map to keep it clean.
		if owner_specific_diagnostics_map.is_empty() {
			info!(
				"[Diag Handler $changeMany] Owner '{}' has no more diagnostics. Removing owner entry from map.",
				owner
			);

			all_diagnostics_map_guard.remove(&owner);
		}

		// Mutex lock released here
	}

	// If any URIs were actually changed for this owner, emit an event.
	if !changed_uri_keys.is_empty() {
		// The payload should indicate which owner's diagnostics changed and for which
		// URIs. Sky can then refetch diagnostics for these URIs or update its view.
		// The actual diagnostic data is NOT sent in this event; Sky should query if
		// needed.
		let event_payload = json!({ "owner": owner, "uris": changed_uri_keys });

		// Keep: Log the event being emitted
		debug!(
			"[Diag Handler $changeMany] Emitting 'diagnostics_changed' event: {:?}",
			event_payload
		);

		if let Err(e) = app.emit_all("diagnostics_changed", event_payload) {
			error!("[Diag Handler $changeMany] Failed to emit 'diagnostics_changed' event: {}", e);
		}
	}

	Ok(Value::Null)
}

/// Handles the `$clear` RPC call from a diagnostics provider.
///
/// Removes all diagnostics associated with the specified `owner` across all
/// resources.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[owner: string]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` with a JSON-RPC error if parsing or state update fails.
pub async fn handle_clear<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let owner = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("diagnostics_clear", "owner", "string", Some(0)))?
		.to_string();

	// Keep: Clearing all is significant
	info!(
		"[Diag Handler] $clear: Attempting to clear all diagnostics for owner='{}'",
		owner
	);

	let app_state = app.state::<AppState>();

	let mut cleared_uri_keys:Vec<String> = Vec::new();

	let owner_had_diagnostics:bool;

	{
		// Scope for AppState.diagnostics_map Mutex lock
		let mut all_diagnostics_map_guard = app_state
			.diagnostics_map
			.lock()
			.map_err(format_diagnostics_lock_error_for_rpc)?;

		// Check if the owner exists and get URIs before removal for notification
		if let Some(owner_specific_diagnostics_map) = all_diagnostics_map_guard.get(&owner) {
			cleared_uri_keys = owner_specific_diagnostics_map.keys().cloned().collect();
		}

		// Attempt to remove the owner's entire entry. `remove` returns Some(value) if
		// key existed.
		owner_had_diagnostics = all_diagnostics_map_guard.remove(&owner).is_some();

		// Mutex lock released here
	}

	if owner_had_diagnostics {
		// Keep: Confirmation log
		info!(
			"[Diag Handler $clear] Cleared all diagnostics for owner '{}'. Affected URIs (if any): {:?}",
			owner, cleared_uri_keys
		);

		// If specific URIs were affected (i.e., the owner had diagnostics for them),

		// notify Sky.
		if !cleared_uri_keys.is_empty() {
			let event_payload = json!({ "owner": owner, "uris": cleared_uri_keys });

			// Keep: Log the event being emitted
			debug!(
				"[Diag Handler $clear] Emitting 'diagnostics_changed' event (due to clear): {:?}",
				event_payload
			);

			if let Err(e) = app.emit_all("diagnostics_changed", event_payload) {
				error!(
					"[Diag Handler $clear] Failed to emit 'diagnostics_changed' event after clear: {}",
					e
				);
			}
		}
	} else {
		// Keep: Warning log if owner wasn't found (idempotent operation, still
		// success)
		warn!(
			"[Diag Handler $clear] Owner '{}' not found in diagnostics map for clearing (no action taken).",
			owner
		);
	}

	Ok(Value::Null)
}

/// Handles the `$getDiagnostics` RPC call from Cocoon.
///
/// Aggregates diagnostics from all owners. If `resource_filter_val` (URI
/// components) is provided, it filters diagnostics to only that resource.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[resource?: UriComponents]`
///   (resource is optional).
///
/// # Returns
/// * `Ok(Value::Array)` of tuples: `[UriComponents, MarkerData[]][]`.
/// * `Err(String)` with a JSON-RPC error if state access or serialization
///   fails.
pub async fn handle_get_diagnostics<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	// Optional UriComponents DTO
	let resource_uri_filter_val = args.get(0);

	// Keep trace for debugging filter issues
	trace!("[Diag Handler] $getDiagnostics: filter='{:?}'", resource_uri_filter_val);

	let app_state = app.state::<AppState>();

	let all_diagnostics_map_guard = app_state
		.diagnostics_map
		.lock()
		.map_err(format_diagnostics_lock_error_for_rpc)?;

	// Parse the filter URI if provided and valid.
	let target_uri_key_filter_opt:Option<String> = resource_uri_filter_val
		 // Ensure it's an object, not just any Value
		.filter(|v| !v.is_null() && v.is_object())
		.and_then(get_uri_key_from_uri_components_dto);

	if resource_uri_filter_val.is_some() && target_uri_key_filter_opt.is_none() {
		warn!(
			"[Diag Handler $getDiagnostics] Invalid or unparsable URI filter provided: {:?}. Proceeding without \
			 filter.",
			resource_uri_filter_val
		);
	}

	// Aggregate diagnostics. Result is Map<UriString, Vec<MarkerData>>
	let mut aggregated_diagnostics_for_response:HashMap<String, Vec<MarkerData>> = HashMap::new();

	for (_owner, owner_specific_diagnostics_map) in all_diagnostics_map_guard.iter() {
		for (uri_key_str, markers_for_uri) in owner_specific_diagnostics_map.iter() {
			// Apply URI filter if one was successfully parsed
			if let Some(target_uri_key) = &target_uri_key_filter_opt {
				if uri_key_str != target_uri_key {
					// Skip if URI doesn't match filter
					continue;
				}
			}

			// Add or extend markers for this URI key
			aggregated_diagnostics_for_response
				.entry(uri_key_str.clone())
				.or_default()
				.extend(markers_for_uri.iter().cloned());
		}
	}

	// Mutex lock released here
	drop(all_diagnostics_map_guard);

	// Convert aggregated map to the expected RPC result format:
	// `[UriComponents, MarkerData[]][]`
	// Ensure UriComponents includes `$mid: 1` as per VS Code DTOs for revival.
	let result_list_dto:Vec<(Value, Vec<MarkerData>)> = aggregated_diagnostics_for_response
		.into_iter()
		.map(|(uri_key_str, markers)| {
			// Reconstruct a minimal UriComponents DTO for the response.
			// Assuming uri_key_str is a full external URI string.
			// TODO: If uri_key_str is not always a full URI, this needs to parse it back
			// into components if Sky expects full UriComponents DTO.
			// For now, sending external URI as 'external' and 'path' (if parsable).
			let (scheme, path) = Url::parse(&uri_key_str)
				.map(|u| (u.scheme().to_string(), u.path().to_string()))
				 // Fallback
				.unwrap_or_else(|_| ("unknown".to_string(), uri_key_str.clone()));

			let uri_components_dto = json!({






				"external": uri_key_str,



				"scheme": scheme,



				"path": path,



				 // Important for VS Code client-side revival
				"$mid": 1
			});

			(uri_components_dto, markers)
		})
		.collect();

	serde_json::to_value(result_list_dto).map_err(|e| {
		error!("[Diag Handler $getDiagnostics] Failed to serialize result list: {}", e);

		error_utils::rpc_error_string(
			format!("Failed to serialize diagnostics result for $getDiagnostics: {}", e),
			Some("ESERIALIZE_DIAG"),
		)
	})
}
