// File: Mountain/Source/Environment/DiagnosticProvider.rs
//
// # Architectural Role: Diagnostic Collection and Aggregation
//
// DiagnosticProvider implements the DiagnosticManager trait, managing diagnostic information
// from multiple sources (language servers, extensions, built-in providers). It aggregates
// diagnostics by owner, file URI, and severity, notifying the UI when changes occur.
//
// # Responsibilities
//
// 1. **Diagnostic Collection**: Maintains collections of diagnostics organized by owner
//    (e.g., TypeScript, Rust, ESLint) and resource URI.
//
// 2. **Diagnostic Aggregation**: Combines diagnostics from multiple sources into a unified
//    view for the user interface.
//
// 3. **Change Notification**: Emits events to the UI (Sky) when diagnostics change,
//    enabling real-time feedback.
//
// 4. **Owner Management**: Allows independent language servers and tools to manage
//    their own diagnostic collections without interference.
//
// 5. **Diagnostic Lifecycle**: Handles setting, updating, and clearing diagnostics
//    for specific resources or entire owner collections.
//
// # Diagnostic Data Model
//
// Diagnostics are stored in ApplicationState.DiagnosticsMap as:
// - Outer map: Owner (String) -> Inner map
// - Inner map: URI String -> Vector of MarkerDataDTO
// - Each MarkerDataDTO represents a single diagnostic with severity, message, range, etc.
//
// # Notification Flow
//
// 1. Language server or extension calls SetDiagnostics(owner, entries)
// 2. Mountain validates and stores diagnostics in ApplicationState
// 3. Mountain identifies changed URIs in this update
// 4. Mountain emits "sky://diagnostics/changed" event with owner and changed URIs
// 5. UI (Sky) receives event and updates diagnostic display
//
// # Patterns Borrowed from VSCode
//
// - **Diagnostic Collections**: Inspired by VSCode's DiagnosticCollection pattern
//   where each language service manages its own collection.
//
// - **Owner Model**: Similar to VSCode's owner concept for distinguishing diagnostic
//   sources (e.g., cs, tslint, eslint).
//
// - **Batch Updates**: Like VSCode, supports setting multiple diagnostics at once
//   for efficiency.
//
// # TODOs
//
// - [ ] Implement diagnostic severity filtering
// - [ ] Add diagnostic code and code description support
// - - [ ] Implement related information support
// - [ ] Add diagnostic tags (deprecated, unnecessary)
// - [ ] Implement diagnostic source tracking
// - [ ] Add support for diagnostic suppression comments
// - [ ] Implement diagnostic cleanup for closed resources
// - [ ] Add diagnostic statistics and metrics
// - [ ] Consider implementing diagnostic versioning for change detection
// - [ ] Add support for diagnostic workspace-wide filtering (exclude files)

use CommonLibrary::{Diagnostic::DiagnosticManager::DiagnosticManager, Error::CommonError::CommonError};
use async_trait::async_trait;
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO;

#[async_trait]
impl DiagnosticManager for MountainEnvironment {
	/// Sets or updates diagnostics for multiple resources from a specific
	/// owner. Empty marker arrays are treated as clearing diagnostics for that URI.
	async fn SetDiagnostics(&self, Owner:String, EntriesDTOValue:Value) -> Result<(), CommonError> {
		info!("[DiagnosticProvider] Setting diagnostics for owner: {}", Owner);

		let DeserializedEntries:Vec<(Value, Option<Vec<MarkerDataDTO>>)> = serde_json::from_value(EntriesDTOValue)
			.map_err(|Error| {
				CommonError::InvalidArgument {
					ArgumentName:"EntriesDTOValue".to_string(),
					Reason:format!("Failed to deserialize diagnostic entries: {}", Error),
				}
			})?;

		let mut DiagnosticsMapGuard = self
			.ApplicationState
			.DiagnosticsMap
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let OwnerMap = DiagnosticsMapGuard.entry(Owner.clone()).or_default();

		let mut ChangedURIKeys = Vec::new();

		for (URIComponentsValue, MarkersOption) in DeserializedEntries {
			let URIKey = Utility::GetURLFromURIComponentsDTO(&URIComponentsValue)?.to_string();

			ChangedURIKeys.push(URIKey.clone());

			if let Some(Markers) = MarkersOption {
				if Markers.is_empty() {
					OwnerMap.remove(&URIKey);
				} else {
					OwnerMap.insert(URIKey, Markers);
				}
			} else {
				OwnerMap.remove(&URIKey);
			}
		}

		drop(DiagnosticsMapGuard);

		// Notify the frontend that diagnostics have changed for specific URIs.
		// Include both added/cleared URIs so UI can update accurately.
		let EventPayload = json!({ "Owner": Owner, "Uris": ChangedURIKeys });

		if let Err(Error) = self.ApplicationHandle.emit("sky://diagnostics/changed", EventPayload) {
			error!("[DiagnosticProvider] Failed to emit 'diagnostics_changed': {}", Error);
		}

		info!("[DiagnosticProvider] Emitted diagnostics changed for {} URI(s)", ChangedURIKeys.len());

		Ok(())
	}

	/// Clears all diagnostics from a specific owner.
	async fn ClearDiagnostics(&self, Owner:String) -> Result<(), CommonError> {
		info!("[DiagnosticProvider] Clearing all diagnostics for owner: {}", Owner);

		let (ClearedCount, ChangedURIKeys): (usize, Vec<String>) = {
			let mut DiagnosticsMapGuard = self
				.ApplicationState
				.DiagnosticsMap
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			DiagnosticsMapGuard
				.remove(&Owner)
			.map(|OwnerMap| {
				let keys: Vec<String> = OwnerMap.keys().cloned().collect();
				(keys.len(), keys)
			})
			.unwrap_or((0, vec![]))
		};

		if !ChangedURIKeys.is_empty() {
			info!("[DiagnosticProvider] Cleared {} diagnostics across {} URI(s)", ClearedCount, ChangedURIKeys.len());

			let EventPayload = json!({ "Owner": Owner, "Uris": ChangedURIKeys });

			if let Err(Error) = self.ApplicationHandle.emit("sky://diagnostics/changed", EventPayload) {
				error!("[DiagnosticProvider] Failed to emit 'diagnostics_changed' on clear: {}", Error);
			}
		}

		Ok(())
	}

	/// Retrieves all diagnostics, optionally filtered by a resource URI.
	/// Returns diagnostics aggregated from all owners for the specified resource(s).
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

		debug!("[DiagnosticProvider] Returning {} diagnostic collection(s)", ResultList.len());

		serde_json::to_value(ResultList).map_err(|Error| CommonError::from(Error))
	}
}
