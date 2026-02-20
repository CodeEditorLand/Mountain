//! # DiagnosticProvider (Environment)
//!
//! Implements the `DiagnosticManager` trait, managing diagnostic information
//! from multiple sources (language servers, extensions, built-in providers). It
//! aggregates diagnostics by owner, file URI, and severity, notifying the UI
//! when changes occur.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Diagnostic Collection
//! - Maintain collections of diagnostics organized by owner (TypeScript, Rust,
//!   ESLint)
//! - Store diagnostics per resource URI for efficient lookup
//! - Support multiple severity levels (Error, Warning, Info, Hint)
//! - Track diagnostic source and code for quick fixes
//!
//! ### 2. Diagnostic Aggregation
//! - Combine diagnostics from multiple sources into unified view
//! - Merge diagnostics for same location from different owners
//! - Sort diagnostics by severity and position
//! - De-duplicate identical diagnostics
//!
//! ### 3. Change Notification
//! - Emit events to UI (Sky) when diagnostics change
//! - Identify changed URIs efficiently for incremental updates
//! - Format diagnostic collections for IPC transmission
//! - Support diagnostic refresh requests
//!
//! ### 4. Owner Management
//! - Allow independent language servers to manage their diagnostics
//! - Support adding/removing diagnostic owners
//! - Prevent interference between different diagnostic sources
//! - Track owner metadata (name, version, etc.)
//!
//! ### 5. Diagnostic Lifecycle
//! - `SetDiagnostics(owner, uri, entries)`: Set diagnostics for owner+URI
//! - `ClearDiagnostics(owner, uri)`: Remove diagnostics
//! - `RemoveOwner(owner)`: Remove all diagnostics from an owner
//! - `GetDiagnostics(uri)`: Retrieve all diagnostics for a URI
//!
//! ## ARCHITECTURAL ROLE
//!
//! DiagnosticProvider is the **diagnostic aggregation hub**:
//!
//! ```text
//! Language Server ──► SetDiagnostics ──► DiagnosticProvider ──► UI Event ──► Sky
//! Extension ──► SetDiagnostics ──► DiagnosticProvider ──► UI Event ──► Sky
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: Error and diagnostic management
//! - Implements `CommonLibrary::Diagnostic::DiagnosticManager` trait
//! - Accessible via `Environment.Require<dyn DiagnosticManager>()`
//!
//! ### Data Storage
//! - `ApplicationState.Feature.Diagnostics`: HashMap<String, HashMap<String,
//! `Vec<MarkerDataDTO>`>>
//!   - Outer key: Owner (e.g., "typescript", "rust-analyzer")
//!   - Inner key: URI string
//!   - Value: Vector of diagnostic markers
//!
//! ### Dependencies
//! - `ApplicationState`: Diagnostic storage
//! - `Log`: Diagnostic change logging
//! - `IPCProvider`: To emit diagnostic change events
//!
//! ### Dependents
//! - Language servers: Report diagnostics via provider
//! - `DispatchLogic`: Route diagnostic-related commands
//! - UI components: Display diagnostics in editor
//!
//! ## DIAGNOSTIC DATA MODEL
//!
//! Each diagnostic is a `MarkerDataDTO`:
//! - `Severity`: Error(8), Warning(4), Information(2), Hint(1)
//! - `Message`: Human-readable description
//! - `StartLineNumber`/`StartColumn`: Start position (0-based)
//! - `EndLineNumber`/`EndColumn`: End position
//! - `Source`: Diagnostic source string (e.g., "tslint")
//! - `Code`: Diagnostic code for quick fix lookup
//! - `ModelVersionIdentifier`: Document version for tracking
//!
//! ## NOTIFICATION FLOW
//!
//! 1. Language server calls `SetDiagnostics(owner, uri, entries)`
//! 2. Provider validates and stores in `ApplicationState.Feature.Diagnostics`
//! 3. Provider identifies which URIs changed in this update
//! 4. Provider emits `sky://diagnostics/changed` event with:
//!    - `owner`: Diagnostic source
//!    - `uris`: List of changed file URIs
//! 5. Sky receives event and requests updated diagnostics for those URIs
//! 6. Sky updates UI (squiggles, Problems panel, etc.)
//!
//! ## ERROR HANDLING
//!
//! - Invalid owner/uri: Logged but operation continues
//! - Empty diagnostic list: Treated as "clear" operation
//! - Serialization errors: Logged and skipped
//! - State lock errors: `CommonError::StateLockPoisoned`
//!
//! ## PERFORMANCE
//!
//! - Diagnostic storage uses nested HashMaps for O(1) lookup
//! - Change detection compares old vs new URI sets
//! - Events are debounced to prevent spam (configurable)
//! - Large diagnostic sets may impact UI responsiveness (consider paging)
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/workbench/services/diagnostic/common/diagnosticCollection.ts` -
//!   Collection management
//! - `vs/platform/diagnostics/common/diagnostics.ts` - Diagnostic data model
//! - `vs/workbench/services/diagnostic/common/diagnosticService.ts` -
//!   Aggregation and events
//!
//! ## TODO
//!
//! - [ ] Implement diagnostic severity filtering (hide certain levels)
//! - [ ] Add diagnostic code actions/quick fixes integration
//! - [ ] Support diagnostic inline messages and hover
//! - [ ] Implement diagnostic history and undo/redo
//! - [ ] Add diagnostic export (to file, clipboard)
//! - [ ] Support diagnostic linting and rule configuration
//! - [ ] Implement diagnostic suppression comments
//! - [ ] Add diagnostic telemetry (frequency, severity distribution)
//! - [ ] Support remote diagnostics (from cloud services)
//! - [ ] Implement diagnostic caching for offline scenarios
//!
//! ## MODULE CONTENTS
//!
//! - `DiagnosticProvider`: Main struct implementing `DiagnosticManager`
//! - Diagnostic storage and retrieval methods
//! - Change notification and event emission
//! - Owner management functions
//! - Diagnostic validation helpers

// 1. **Diagnostic Collection**: Maintains collections of diagnostics organized
//    by owner (e.g., TypeScript, Rust, ESLint) and resource URI.
//
// 2. **Diagnostic Aggregation**: Combines diagnostics from multiple sources
//    into a unified view for the user interface.
//
// 3. **Change Notification**: Emits events to the UI (Sky) when diagnostics
//    change, enabling real-time feedback.
//
// 4. **Owner Management**: Allows independent language servers and tools to
//    manage their own diagnostic collections without interference.
//
// 5. **Diagnostic Lifecycle**: Handles setting, updating, and clearing
//    diagnostics for specific resources or entire owner collections.
//
// # Diagnostic Data Model
//
// Diagnostics are stored in ApplicationState.Feature.Diagnostics as:
// - Outer map: Owner (String) -> Inner map
// - Inner map: URI String -> Vector of MarkerDataDTO
// - Each MarkerDataDTO represents a single diagnostic with severity, message,
//   range, etc.
//
// # Notification Flow
//
// 1. Language server or extension calls SetDiagnostics(owner, entries)
// 2. Mountain validates and stores diagnostics in ApplicationState
// 3. Mountain identifies changed URIs in this update
// 4. Mountain emits "sky://diagnostics/changed" event with owner and changed
//    URIs
// 5. UI (Sky) receives event and updates diagnostic display
//
// # Patterns Borrowed from VSCode
//
// - **Diagnostic Collections**: Inspired by VSCode's DiagnosticCollection
//   pattern where each language service manages its own collection.
//
// - **Owner Model**: Similar to VSCode's owner concept for distinguishing
//   diagnostic sources (e.g., cs, tslint, eslint).
//
// - **Batch Updates**: Like VSCode, supports setting multiple diagnostics at
//   once for efficiency.
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
	/// owner. Empty marker arrays are treated as clearing diagnostics for that
	/// URI.
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
			.Feature
			.Diagnostics
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

		info!(
			"[DiagnosticProvider] Emitted diagnostics changed for {} URI(s)",
			ChangedURIKeys.len()
		);

		Ok(())
	}

	/// Clears all diagnostics from a specific owner.
	async fn ClearDiagnostics(&self, Owner:String) -> Result<(), CommonError> {
		info!("[DiagnosticProvider] Clearing all diagnostics for owner: {}", Owner);

		let (ClearedCount, ChangedURIKeys):(usize, Vec<String>) = {
			let mut DiagnosticsMapGuard = self
				.ApplicationState
				.Feature
				.Diagnostics
				.DiagnosticsMap
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			DiagnosticsMapGuard
				.remove(&Owner)
				.map(|OwnerMap| {
					let keys:Vec<String> = OwnerMap.keys().cloned().collect();
					(keys.len(), keys)
				})
				.unwrap_or((0, vec![]))
		};

		if !ChangedURIKeys.is_empty() {
			info!(
				"[DiagnosticProvider] Cleared {} diagnostics across {} URI(s)",
				ClearedCount,
				ChangedURIKeys.len()
			);

			let EventPayload = json!({ "Owner": Owner, "Uris": ChangedURIKeys });

			if let Err(Error) = self.ApplicationHandle.emit("sky://diagnostics/changed", EventPayload) {
				error!("[DiagnosticProvider] Failed to emit 'diagnostics_changed' on clear: {}", Error);
			}
		}

		Ok(())
	}

	/// Retrieves all diagnostics, optionally filtered by a resource URI.
	/// Returns diagnostics aggregated from all owners for the specified
	/// resource(s).
	async fn GetAllDiagnostics(&self, ResourceURIFilterOption:Option<Value>) -> Result<Value, CommonError> {
		debug!(
			"[DiagnosticProvider] Getting all diagnostics with filter: {:?}",
			ResourceURIFilterOption
		);

		let DiagnosticsMapGuard = self
			.ApplicationState
			.Feature
			.Diagnostics
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
