// ---------------------------------------------------------------------------------------------
// Mountain Diagnostics Handlers (handlers/diagnostics.rs)
// --------------------------------------------------------------------------------------------
// Manages diagnostic information (problems/markers).
// - RPC handlers (`handle_change_many`, etc.) are called by Track, create
//   effects, and run them via AppRuntime.
// - Effect logic handlers (`handle_*_effect_logic`) are called by
//   MountainEnvironment's DiagnosticsManager implementation, contain the core
//   logic for updating AppState and emitting events.
// --------------------------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::MutexGuard as StdMutexGuard; // For type in map_app_state_lock_error

// Common DTOs and effect constructors
use Land_Common::{
	diagnostics_effects::{self, MarkerDataDto as CommonMarkerDataDto}, // DTO from Land_Common
	errors::CommonError,
};
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime as TauriRuntime, State}; // Added State

use crate::{
	app_state::{AppState, MarkerData as AppStateMarkerData}, // Mountain's internal MarkerData
	handlers::error_utils,
	runtime::AppRuntime, // For running effects
};

// --- Helper Functions ---

/// Helper to get a consistent string key (typically the `external` URI string)
/// from a `serde_json::Value` representing `UriComponents` DTO.
/// Public within the crate for use by `environment/diagnostics_provider.rs` if
/// needed, or primarily used by `handle_set_diagnostics_effect_logic`.
pub(crate) fn get_uri_key_from_uri_components_dto_for_diag(uri_components:&Value) -> Option<String> {
	if let Some(ext_uri_str) = uri_components.get("external").and_then(Value::as_str) {
		return Some(ext_uri_str.to_string());
	}
	warn!(
		"[Diag Helper] URI DTO missing 'external', attempting fallback. DTO: {:?}",
		uri_components
	);
	let scheme = uri_components.get("scheme").and_then(Value::as_str)?;
	let path = uri_components.get("path").and_then(Value::as_str)?;
	let authority = uri_components.get("authority").and_then(Value::as_str).unwrap_or("");
	Some(format!("{}://{}{}", scheme, authority, path))
}

fn map_app_state_lock_error<T>(e:std::sync::PoisonError<StdMutexGuard<'_, T>>, context:&str) -> CommonError {
	let msg = format!("[Diag Handler LockErr] Failed lock on {}: {}", context, e);
	error!("{}", msg);
	CommonError::StateLock(msg)
}

// --- RPC Handlers (Called by Track - create and run effects) ---

pub async fn handle_change_many<R:TauriRuntime>(
	_app_handle:AppHandle<R>, // AppHandle available via runtime if needed by effect indirectly
	runtime:State<'_, Arc<AppRuntime>>,
	args:Value,
) -> Result<Value, String> {
	let owner = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("diagnostics_changeMany", "owner", "string", Some(0)))?
		.to_string();
	let entries_dto_val = args.get(1).cloned().ok_or_else(|| {
		error_utils::rpc_param_error_string("diagnostics_changeMany", "entries_dto_val", "array", Some(1))
	})?;

	info!(
		"[Diag RPC Handler] $changeMany: owner='{}', {} entries. Dispatching effect.",
		owner,
		entries_dto_val.as_array().map_or(0, |a| a.len())
	);

	let effect = diagnostics_effects::set_diagnostics(owner.clone(), entries_dto_val);
	runtime.run(effect).await
        .map(|_| Value::Null) // Effect returns Result<(), CommonError>
        .map_err(|e| error_utils::map_common_error_to_rpc_string(e, &format!("set_diagnostics for {}", owner)))
}

pub async fn handle_clear<R:TauriRuntime>(
	_app_handle:AppHandle<R>,
	runtime:State<'_, Arc<AppRuntime>>,
	args:Value,
) -> Result<Value, String> {
	let owner = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("diagnostics_clear", "owner", "string", Some(0)))?
		.to_string();
	info!("[Diag RPC Handler] $clear: owner='{}'. Dispatching effect.", owner);

	let effect = diagnostics_effects::clear_diagnostics(owner.clone());
	runtime
		.run(effect)
		.await
		.map(|_| Value::Null)
		.map_err(|e| error_utils::map_common_error_to_rpc_string(e, &format!("clear_diagnostics for {}", owner)))
}

pub async fn handle_get_diagnostics<R:TauriRuntime>(
	_app_handle:AppHandle<R>,
	runtime:State<'_, Arc<AppRuntime>>,
	args:Value,
) -> Result<Value, String> {
	let resource_uri_filter_opt = args.get(0).cloned(); // Optional Value for UriComponents
	trace!(
		"[Diag RPC Handler] $getDiagnostics: filter='{:?}'. Dispatching effect.",
		resource_uri_filter_opt
	);

	let effect = diagnostics_effects::get_all_diagnostics(resource_uri_filter_opt);
	runtime.run(effect).await // Effect returns Result<Value, CommonError>
        .map_err(|e| error_utils::map_common_error_to_rpc_string(e, "get_all_diagnostics"))
}

// --- Effect Logic Handlers (Called by MountainEnvironment's DiagnosticsManager
// impl) ---

pub async fn handle_set_diagnostics_effect_logic<R:TauriRuntime>(
	app_handle:AppHandle<R>,
	owner:String,
	entries_dto_val:Value, // Expected: Array of [UriComponentsValue, Option<Vec<CommonMarkerDataDtoAsValue>>]
) -> Result<(), CommonError> {
	info!(
		"[Diag EffectLogic] SetDiagnostics: Owner='{}', NumEntries={}",
		owner,
		entries_dto_val.as_array().map_or(0, |a| a.len())
	);
	trace!("[Diag EffectLogic] SetDiagnostics Full DTO: {:?}", entries_dto_val);

	// Deserialize entries: Vec<[UriComponentsValue (Value),
	// Option<Vec<CommonMarkerDataDto>>]>
	let entries_deserialized:Vec<(Value, Option<Vec<CommonMarkerDataDto>>)> =
		serde_json::from_value(entries_dto_val.clone()).map_err(|e| {
			CommonError::InvalidArg(
				"entries_dto_val".to_string(),
				format!("Failed to deserialize diagnostic entries: {}", e),
			)
		})?;

	let app_state = app_handle.state::<AppState>();
	let mut changed_uri_keys_for_event:Vec<String> = Vec::new();

	{
		// Scope for diagnostics_map lock
		let mut diag_map_guard = app_state
			.diagnostics_map
			.lock()
			.map_err(|e| map_app_state_lock_error(e, "diagnostics_map for set"))?;
		let owner_map = diag_map_guard.entry(owner.clone()).or_default();

		for (uri_components_val, markers_opt_common_dto) in entries_deserialized {
			let uri_key = get_uri_key_from_uri_components_dto_for_diag(&uri_components_val).ok_or_else(|| {
				CommonError::InvalidArg("uri_components_val".to_string(), "Invalid URI in diagnostic entry".to_string())
			})?;

			if !changed_uri_keys_for_event.contains(&uri_key) {
				changed_uri_keys_for_event.push(uri_key.clone());
			}

			if let Some(common_markers_dto_vec) = markers_opt_common_dto {
				if !common_markers_dto_vec.is_empty() {
					// Convert CommonMarkerDataDto to AppStateMarkerData
					let app_state_markers:Vec<AppStateMarkerData> = common_markers_dto_vec
						.into_iter()
						.map(|common_dto| {
							// TODO: Implement a proper From<CommonMarkerDataDto> for AppStateMarkerData
							// or a conversion function if fields differ significantly.
							// For now, assuming direct field compatibility or simple serde roundtrip if
							// identical. This is a placeholder conversion.
							serde_json::from_value(serde_json::to_value(common_dto).unwrap_or(Value::Null))
								.unwrap_or_else(|e| {
									warn!(
										"[Diag EffectLogic] Failed to convert CommonMarkerDataDto to \
										 AppStateMarkerData: {}. Using default.",
										e
									);
									Default::default() // Or skip this marker
								})
						})
						.collect();
					trace!(
						"[Diag EffectLogic] Setting {} markers for owner '{}', URI '{}'",
						app_state_markers.len(),
						owner,
						uri_key
					);
					owner_map.insert(uri_key, app_state_markers);
				} else {
					// Empty array means clear for this URI
					trace!(
						"[Diag EffectLogic] Clearing markers for owner '{}', URI '{}' (empty array).",
						owner, uri_key
					);
					owner_map.remove(&uri_key);
				}
			} else {
				// None means clear for this URI
				trace!(
					"[Diag EffectLogic] Clearing markers for owner '{}', URI '{}' (null markers).",
					owner, uri_key
				);
				owner_map.remove(&uri_key);
			}
		}
		if owner_map.is_empty() {
			info!("[Diag EffectLogic] Owner '{}' has no diagnostics. Removing entry.", owner);
			diag_map_guard.remove(&owner);
		}
	} // Lock released

	if !changed_uri_keys_for_event.is_empty() {
		let event_payload = json!({ "owner": owner, "uris": changed_uri_keys_for_event });
		debug!("[Diag EffectLogic] Emitting 'diagnostics_changed' event: {:?}", event_payload);
		if let Err(e) = app_handle.emit("diagnostics_changed", event_payload) {
			error!("[Diag EffectLogic] Failed to emit 'diagnostics_changed': {}", e);
		}
	}
	Ok(())
}

pub async fn handle_clear_diagnostics_effect_logic<R:TauriRuntime>(
	app_handle:AppHandle<R>,
	owner:String,
) -> Result<(), CommonError> {
	info!("[Diag EffectLogic] ClearDiagnostics: Owner='{}'", owner);
	let app_state = app_handle.state::<AppState>();
	let mut cleared_uri_keys_for_event:Vec<String> = Vec::new();
	let owner_existed_and_had_diagnostics:bool;

	{
		let mut diag_map_guard = app_state
			.diagnostics_map
			.lock()
			.map_err(|e| map_app_state_lock_error(e, "diagnostics_map for clear"))?;
		if let Some(owner_map) = diag_map_guard.get(&owner) {
			if !owner_map.is_empty() {
				// Only consider it "cleared" if there was something to clear for URIs
				cleared_uri_keys_for_event = owner_map.keys().cloned().collect();
			}
		}
		owner_existed_and_had_diagnostics =
			diag_map_guard.remove(&owner).is_some() && !cleared_uri_keys_for_event.is_empty();
	}

	if owner_existed_and_had_diagnostics {
		info!(
			"[Diag EffectLogic] Cleared diagnostics for owner '{}'. Affected URIs: {:?}",
			owner, cleared_uri_keys_for_event
		);
		let event_payload = json!({ "owner": owner, "uris": cleared_uri_keys_for_event });
		debug!("[Diag EffectLogic] Emitting 'diagnostics_changed' (clear): {:?}", event_payload);
		if let Err(e) = app_handle.emit("diagnostics_changed", event_payload) {
			error!("[Diag EffectLogic] Failed to emit 'diagnostics_changed' after clear: {}", e);
		}
	} else {
		warn!("[Diag EffectLogic] Owner '{}' not found or had no diagnostics to clear.", owner);
	}
	Ok(())
}

pub async fn handle_get_all_diagnostics_effect_logic<R:TauriRuntime>(
	app_handle:AppHandle<R>,
	resource_uri_filter_opt:Option<Value>,
) -> Result<Value, CommonError> {
	trace!("[Diag EffectLogic] GetAllDiagnostics: filter='{:?}'", resource_uri_filter_opt);
	let app_state = app_handle.state::<AppState>();
	let all_diagnostics_map_guard = app_state
		.diagnostics_map
		.lock()
		.map_err(|e| map_app_state_lock_error(e, "diagnostics_map for get_all"))?;

	let target_uri_key_filter_opt:Option<String> = resource_uri_filter_opt
		.filter(|v| !v.is_null() && v.is_object())
		.and_then(|v| get_uri_key_from_uri_components_dto_for_diag(&v));

	if resource_uri_filter_opt.is_some() && target_uri_key_filter_opt.is_none() {
		warn!(
			"[Diag EffectLogic] Invalid URI filter for GetAllDiagnostics: {:?}. Ignoring filter.",
			resource_uri_filter_opt
		);
	}

	let mut aggregated_diagnostics_for_response:HashMap<String, Vec<CommonMarkerDataDto>> = HashMap::new();

	for (_owner, owner_map) in all_diagnostics_map_guard.iter() {
		for (uri_key_str, app_state_markers_vec) in owner_map.iter() {
			if let Some(ref filter_key) = target_uri_key_filter_opt {
				if uri_key_str != filter_key {
					continue;
				}
			}
			let common_markers_dto_vec:Vec<CommonMarkerDataDto> = app_state_markers_vec
				.iter()
				.map(|app_state_marker| {
					// TODO: Implement proper From<AppStateMarkerData> for CommonMarkerDataDto or
					// conversion. Placeholder conversion:
					serde_json::from_value(serde_json::to_value(app_state_marker).unwrap_or(Value::Null))
						.unwrap_or_else(|e| {
							warn!(
								"[Diag EffectLogic] Failed to convert AppStateMarkerData to CommonMarkerDataDto for \
								 URI {}: {}. Using default.",
								uri_key_str, e
							);
							// Return a default or skip. For now, a default to indicate issue.
							CommonMarkerDataDto {
								severity:8,
								message:"Conversion Error".into(),
								start_line_number:1,
								start_column:1,
								end_line_number:1,
								end_column:1,
								..Default::default()
							}
						})
				})
				.collect();

			aggregated_diagnostics_for_response
				.entry(uri_key_str.clone())
				.or_default()
				.extend(common_markers_dto_vec);
		}
	}
	drop(all_diagnostics_map_guard);

	// Convert to RPC format: Vec<[UriComponentsValue,
	// Vec<CommonMarkerDataDtoAsValue>]>
	let result_list_dto:Vec<(Value, Vec<CommonMarkerDataDto>)> = aggregated_diagnostics_for_response
		.into_iter()
		.map(|(uri_key_str, markers)| {
			let (scheme, path) = url::Url::parse(&uri_key_str)
				.map(|u| (u.scheme().to_string(), u.path().to_string()))
				.unwrap_or_else(|_| ("unknown".to_string(), uri_key_str.clone()));
			let uri_components_dto = json!({ "external": uri_key_str, "scheme": scheme, "path": path, "$mid": 1 });
			(uri_components_dto, markers)
		})
		.collect();

	serde_json::to_value(result_list_dto).map_err(|e| {
		error!(
			"[Diag EffectLogic] Failed to serialize result list for GetAllDiagnostics: {}",
			e
		);
		CommonError::SerdeError { description:format!("Failed to serialize diagnostics result: {}", e) }
	})
}
