// @module DiagnosticLogic
// @description Contains the core logic for managing diagnostic collections.
// This includes storing diagnostics from various sources and notifying the User Interface
// of changes.

use Common::{error::CommonError, language_feature::DTO::MarkerDataDTO};
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	Environment::Utility::{self, GetUrlFromUriDTO},
};

// Logic to set or update diagnostics for multiple resources from a specific
// owner.
pub async fn SetDiagnosticsLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	owner:String,
	entries_DTO_value:Value,
) -> Result<(), CommonError> {
	info!("[DiagnosticLogic] Setting diagnostics for owner: {}", owner);
	let app_state = app_handle.state::<ApplicationState>();
	let mut changed_uri_keys = Vec::new();

	// The payload is an array of [UriComponents, MarkerDataDTO[] | null]
	let deserialized_entries:Vec<(Value, Option<Vec<MarkerDataDTO>>)> = serde_json::from_value(entries_DTO_value)
		.map_err(|e| {
			CommonError::InvalidArg {
				ArgumentName:"EntriesDTOValue".to_string(),
				Reason:format!("Failed to deserialize diagnostic entries: {}", e),
			}
		})?;

	let mut diag_map_guard = app_state
		.DiagnosticsMap
		.lock()
		.map_err(Utility::MapAppStateLockErrorToCommonError)?;
	let owner_map = diag_map_guard.entry(owner.clone()).or_default();

	for (uri_components_val, markers_opt) in deserialized_entries {
		let uri_key = GetUrlFromUriDTO(&uri_components_val)?.to_string();

		if let Some(markers) = markers_opt {
			owner_map.insert(uri_key.clone(), markers);
		} else {
			owner_map.remove(&uri_key);
		}
		changed_uri_keys.push(uri_key);
	}
	drop(diag_map_guard);

	// Notify the frontend that diagnostics have changed for specific URIs.
	let event_payload = json!({ "Owner": owner, "Uris": changed_uri_keys });
	if let Err(e) = app_handle.emit("sky://diagnostics/changed", event_payload) {
		error!("[DiagnosticLogic] Failed to emit 'diagnostics_changed': {}", e);
	}

	Ok(())
}

// Logic to clear all diagnostics from a specific owner.
pub async fn ClearDiagnosticsLogic<R:Runtime>(app_handle:&AppHandle<R>, owner:String) -> Result<(), CommonError> {
	info!("[DiagnosticLogic] Clearing all diagnostics for owner: {}", owner);
	let app_state = app_handle.state::<ApplicationState>();
	let mut diag_map_guard = app_state
		.DiagnosticsMap
		.lock()
		.map_err(Utility::MapAppStateLockErrorToCommonError)?;

	if let Some(owner_map) = diag_map_guard.remove(&owner) {
		let changed_uri_keys:Vec<String> = owner_map.keys().cloned().collect();
		drop(diag_map_guard);

		let event_payload = json!({ "Owner": owner, "Uris": changed_uri_keys });
		if let Err(e) = app_handle.emit("sky://diagnostics/changed", event_payload) {
			error!("[DiagnosticLogic] Failed to emit 'diagnostics_changed' on clear: {}", e);
		}
	}
	Ok(())
}

// Logic to retrieve all diagnostics, optionally filtered by a resource URI.
pub async fn GetAllDiagnosticsLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	resource_uri_filter:Option<Value>,
) -> Result<Value, CommonError> {
	debug!(
		"[DiagnosticLogic] Getting all diagnostics with filter: {:?}",
		resource_uri_filter
	);
	let app_state = app_handle.state::<ApplicationState>();
	let diag_map_guard = app_state
		.DiagnosticsMap
		.lock()
		.map_err(Utility::MapAppStateLockErrorToCommonError)?;

	let mut result_list:Vec<(String, Vec<MarkerDataDTO>)> = Vec::new();

	if let Some(filter_uri_value) = resource_uri_filter {
		let filter_uri_key = GetUrlFromUriDTO(&filter_uri_value)?.to_string();

		for owner_map in diag_map_guard.values() {
			if let Some(markers) = owner_map.get(&filter_uri_key) {
				// In this case, we just return the markers for the specific file.
				// A better API might be needed, but this matches the likely intent.
				result_list.push((filter_uri_key.clone(), markers.clone()));
			}
		}
	} else {
		// Aggregate all diagnostics from all owners.
		let mut aggregated_by_uri:std::collections::HashMap<String, Vec<MarkerDataDTO>> =
			std::collections::HashMap::new();
		for owner_map in diag_map_guard.values() {
			for (uri_key, markers) in owner_map.iter() {
				aggregated_by_uri.entry(uri_key.clone()).or_default().extend(markers.clone());
			}
		}
		result_list = aggregated_by_uri.into_iter().collect();
	}

	serde_json::to_value(result_list).map_err(|e| CommonError::SerdeError { Description:e.to_string() })
}
