use Common::{
	error::CommonError,
	language_feature::dto::MarkerDataDto, // Assuming DTO is in language_feature
};
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// @module DiagnosticsLogic
/// @description Contains the core logic for managing diagnostic collections.
/// This includes storing diagnostics from various sources and notifying the UI
/// of changes.
use crate::{AppState::AppState::AppState, environment::Utils, handlers::error_utils};

/// Logic to set or update diagnostics for multiple resources from a specific
/// owner.
pub async fn SetDiagnosticsLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Owner:String,
	EntriesDtoValue:Value,
) -> Result<(), CommonError> {
	info!("[DiagnosticsLogic] Setting diagnostics for owner: {}", Owner);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut ChangedUriKeys = Vec::new();

	// The payload is an array of [UriComponents, MarkerDataDto[] | null]
	let DeserializedEntries:Vec<(Value, Option<Vec<MarkerDataDto>>)> = serde_json::from_value(EntriesDtoValue)
		.map_err(|e| {
			CommonError::InvalidArg {
				ArgumentName:"EntriesDtoValue".to_string(),
				Reason:format!("Failed to deserialize diagnostic entries: {}", e),
			}
		})?;

	let mut DiagMapGuard = AppStateInstance
		.DiagnosticsMap
		.lock()
		.map_err(Utils::MapAppStateLockErrorToCommonError)?;
	let OwnerMap = DiagMapGuard.entry(Owner.clone()).or_default();

	for (UriComponentsVal, MarkersOpt) in DeserializedEntries {
		// Use the URI's external representation as the key for consistency.
		let UriKey = UriComponentsVal
			.get("external")
			.and_then(Value::as_str)
			.ok_or_else(|| {
				CommonError::InvalidArg {
					ArgumentName:"UriKey".to_string(),
					Reason:"URI in diagnostic entry is missing 'external' field.".to_string(),
				}
			})?
			.to_string();

		if let Some(Markers) = MarkersOpt {
			OwnerMap.insert(UriKey.clone(), Markers);
		} else {
			OwnerMap.remove(&UriKey);
		}
		ChangedUriKeys.push(UriKey);
	}
	drop(DiagMapGuard);

	// Notify the frontend that diagnostics have changed for specific URIs.
	let EventPayload = json!({ "Owner": Owner, "Uris": ChangedUriKeys });
	if let Err(e) = AppHandle.emit("sky://diagnostics/changed", EventPayload) {
		error!("[DiagnosticsLogic] Failed to emit 'diagnostics_changed': {}", e);
	}

	Ok(())
}

/// Logic to clear all diagnostics from a specific owner.
pub async fn ClearDiagnosticsLogic<R:Runtime>(AppHandle:&AppHandle<R>, Owner:String) -> Result<(), CommonError> {
	info!("[DiagnosticsLogic] Clearing all diagnostics for owner: {}", Owner);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut DiagMapGuard = AppStateInstance
		.DiagnosticsMap
		.lock()
		.map_err(Utils::MapAppStateLockErrorToCommonError)?;

	if let Some(OwnerMap) = DiagMapGuard.remove(&Owner) {
		let ChangedUriKeys:Vec<String> = OwnerMap.keys().cloned().collect();
		drop(DiagMapGuard);

		let EventPayload = json!({ "Owner": Owner, "Uris": ChangedUriKeys });
		if let Err(e) = AppHandle.emit("sky://diagnostics/changed", EventPayload) {
			error!("[DiagnosticsLogic] Failed to emit 'diagnostics_changed' on clear: {}", e);
		}
	}
	Ok(())
}

/// Logic to retrieve all diagnostics, optionally filtered by a resource URI.
pub async fn GetAllDiagnosticsLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	ResourceUriFilterOption:Option<Value>,
) -> Result<Value, CommonError> {
	debug!(
		"[DiagnosticsLogic] Getting all diagnostics with filter: {:?}",
		ResourceUriFilterOption
	);
	let AppStateInstance = AppHandle.state::<AppState>();
	let DiagMapGuard = AppStateInstance
		.DiagnosticsMap
		.lock()
		.map_err(Utils::MapAppStateLockErrorToCommonError)?;

	let mut ResultList:Vec<(String, Vec<MarkerDataDto>)> = Vec::new();

	if let Some(FilterUriValue) = ResourceUriFilterOption {
		let FilterUriKey = FilterUriValue.get("external").and_then(Value::as_str).ok_or_else(|| {
			CommonError::InvalidArg {
				ArgumentName:"ResourceUriFilter".to_string(),
				Reason:"Filter URI is missing 'external' field.".to_string(),
			}
		})?;

		for OwnerMap in DiagMapGuard.values() {
			if let Some(Markers) = OwnerMap.get(FilterUriKey) {
				// In this case, we just return the markers for the specific file.
				// A better API might be needed, but this matches the likely intent.
				ResultList.push((FilterUriKey.to_string(), Markers.clone()));
			}
		}
	} else {
		// Aggregate all diagnostics from all owners.
		let mut AggregatedByUri:std::collections::HashMap<String, Vec<MarkerDataDto>> =
			std::collections::HashMap::new();
		for OwnerMap in DiagMapGuard.values() {
			for (UriKey, Markers) in OwnerMap.iter() {
				AggregatedByUri.entry(UriKey.clone()).or_default().extend(Markers.clone());
			}
		}
		ResultList = AggregatedByUri.into_iter().collect();
	}

	serde_json::to_value(ResultList).map_err(|e| CommonError::SerdeError { Description:e.to_string() })
}
