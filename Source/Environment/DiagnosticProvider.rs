// File: Mountain/Source/Environment/DiagnosticProvider.rs
// Role: Implements the `DiagnosticManager` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Manage diagnostic collections in the central `ApplicationState`.
//   - Store diagnostics from various sources (owners).
//   - Notify the UI (`Sky`) of changes to diagnostics.
//   - Provide an aggregated view of all diagnostics.

//! # DiagnosticProvider Implementation
//!
//! Implements the `DiagnosticManager` trait for the `MountainEnvironment`. This
//! provider contains the core logic for managing diagnostic collections,

//! including storing diagnostics from various sources and notifying the UI of
//! changes.

use Common::{Diagnostic::DiagnosticManager::DiagnosticManager, Error::CommonError::CommonError};
use async_trait::async_trait;
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO;

#[async_trait]
impl DiagnosticManager for MountainEnvironment {
	/// Sets or updates diagnostics for multiple resources from a specific
	/// owner.
	async fn SetDiagnostics(&self, Owner:String, EntriesDTOValue:Value) -> Result<(), CommonError> {
		info!("[DiagnosticProvider] Setting diagnostics for owner: {}", Owner);

		let mut ChangedURIKeys = Vec::new();

		let DeserializedEntries:Vec<(Value, Option<Vec<MarkerDataDTO>>)> = serde_json::from_value(EntriesDTOValue)
			.map_err(|e| {
				CommonError::InvalidArgument {
					ArgumentName:"EntriesDTOValue".to_string(),

					Reason:format!("Failed to deserialize diagnostic entries: {}", e),
				}
			})?;

		let mut DiagnosticsMapGuard = self
			.ApplicationState
			.DiagnosticsMap
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let OwnerMap = DiagnosticsMapGuard.entry(Owner.clone()).or_default();

		for (URIComponentsValue, MarkersOption) in DeserializedEntries {
			let URIKey = Utility::GetURLFromURIComponentsDTO(&URIComponentsValue)?.to_string();

			ChangedURIKeys.push(URIKey.clone());

			if let Some(Markers) = MarkersOption {
				if Markers.is_empty() {
					OwnerMap.remove(&URIKey);
				} else {
					OwnerMap.insert(URIKey.clone(), Markers);
				}
			} else {
				OwnerMap.remove(&URIKey);
			}
		}

		drop(DiagnosticsMapGuard);

		// Notify the frontend that diagnostics have changed for specific URIs.
		let EventPayload = json!({ "Owner": Owner, "Uris": ChangedURIKeys });

		if let Err(e) = self.ApplicationHandle.emit("sky://diagnostics/changed", EventPayload) {
			error!("[DiagnosticProvider] Failed to emit 'diagnostics_changed': {}", e);
		}

		Ok(())
	}

	/// Clears all diagnostics from a specific owner.
	async fn ClearDiagnostics(&self, Owner:String) -> Result<(), CommonError> {
		info!("[DiagnosticProvider] Clearing all diagnostics for owner: {}", Owner);

		let mut DiagnosticsMapGuard = self
			.ApplicationState
			.DiagnosticsMap
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(OwnerMap) = DiagnosticsMapGuard.remove(&Owner) {
			let ChangedURIKeys:Vec<String> = OwnerMap.keys().cloned().collect();

			drop(DiagnosticsMapGuard);

			if !ChangedURIKeys.is_empty() {
				let EventPayload = json!({ "Owner": Owner, "Uris": ChangedURIKeys });

				if let Err(e) = self.ApplicationHandle.emit("sky://diagnostics/changed", EventPayload) {
					error!("[DiagnosticProvider] Failed to emit 'diagnostics_changed' on clear: {}", e);
				}
			}
		}

		Ok(())
	}

	/// Retrieves all diagnostics, optionally filtered by a resource URI.
	async fn GetAllDiagnostics(&self, ResourceURIFilterOption:Option<Value>) -> Result<Value, CommonError> {
		debug!(
			"[DiagnosticProvider] Getting all diagnostics with filter: {:?}",
			ResourceURIFilterOption
		);

		let DiagnosticsMapGuard = self
			.ApplicationState
			.DiagnosticsMap
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let mut ResultMap:std::collections::HashMap<String, Vec<MarkerDataDTO>> = std::collections::HashMap::new();

		if let Some(FilterURIValue) = ResourceURIFilterOption {
			let FilterURIKey = Utility::GetURLFromURIComponentsDTO(&FilterURIValue)?.to_string();

			for OwnerMap in DiagnosticsMapGuard.values() {
				if let Some(Markers) = OwnerMap.get(&FilterURIKey) {
					ResultMap.entry(FilterURIKey.clone()).or_default().extend(Markers.clone());
				}
			}
		} else {
			// Aggregate all diagnostics from all owners for all files.
			for OwnerMap in DiagnosticsMapGuard.values() {
				for (URIKey, Markers) in OwnerMap.iter() {
					ResultMap.entry(URIKey.clone()).or_default().extend(Markers.clone());
				}
			}
		}

		let ResultList:Vec<(String, Vec<MarkerDataDTO>)> = ResultMap.into_iter().collect();

		serde_json::to_value(ResultList).map_err(|e| CommonError::SerializationError { Description:e.to_string() })
	}
}
