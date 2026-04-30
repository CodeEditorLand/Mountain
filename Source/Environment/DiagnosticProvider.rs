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
//! - `StartLineNumber`/`StartColumn`: Start position (1-based, matches
//!   workbench `IMarkerData` - Cocoon's `LanguagesNamespace.ts`
//!   `NormaliseDiagnostic` adds the `+ 1` from vscode.Position 0-based before
//!   sending to Mountain)
//! - `EndLineNumber`/`EndColumn`: End position (1-based, same convention)
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

use CommonLibrary::{
	Diagnostic::DiagnosticManager::DiagnosticManager,
	Error::CommonError::CommonError,
	IPC::SkyEvent::SkyEvent,
};
use async_trait::async_trait;
use serde_json::{Value, json};

// `tauri::Emitter` is no longer used directly here - all emits
// route through `LogSkyEmit` which carries the trait import. The
// import was previously here for the direct `.emit()` calls now
// replaced. Removed to keep the file warning-clean.
use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, IPC::SkyEmit::LogSkyEmit, dev_log};

#[async_trait]
impl DiagnosticManager for MountainEnvironment {
	/// Sets or updates diagnostics for multiple resources from a specific
	/// owner. Empty marker arrays are treated as clearing diagnostics for that
	/// URI.
	async fn SetDiagnostics(&self, Owner:String, EntriesDTOValue:Value) -> Result<(), CommonError> {
		dev_log!("extensions", "[DiagnosticProvider] Setting diagnostics for owner: {}", Owner);

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
		// `ChangedEntries` carries the post-update marker set per URI so the
		// Sky-side `cel:diagnostics:changed` listener can call
		// `IMarkerService.changeOne(owner, uri, markers)` without an extra
		// IPC round-trip per change. URIs whose markers were cleared still
		// appear here with an empty array, so the workbench replaces the
		// previous owner-set rather than leaving stale red squiggles.
		let mut ChangedEntries:Vec<serde_json::Value> = Vec::new();

		for (URIComponentsValue, MarkersOption) in DeserializedEntries {
			// Per-entry tolerance: a single malformed URI (extension
			// passed an empty `.path`, exotic scheme, or non-string
			// authority) used to fail the entire batch via `?`-prop -
			// dropping every well-formed diagnostic in the same call
			// because of one bad sibling. Mirror VS Code's
			// `MarkerService._toMarker` which returns `undefined` for
			// bad entries instead of throwing: skip the offender, log
			// once, keep going so the rest of the batch reaches the
			// renderer.
			let URIKey = match Utility::GetURLFromURIComponentsDTO(&URIComponentsValue) {
				Ok(Url) => Url.to_string(),
				Err(Error) => {
					dev_log!(
						"extensions",
						"warn: [DiagnosticProvider] skipping diagnostic entry with bad URI: {} (raw={:?})",
						Error,
						URIComponentsValue
					);
					continue;
				},
			};
			if URIKey.is_empty() {
				dev_log!(
					"extensions",
					"warn: [DiagnosticProvider] skipping diagnostic entry with empty URI string"
				);
				continue;
			}

			ChangedURIKeys.push(URIKey.clone());

			let MarkersForEvent = match MarkersOption {
				Some(Markers) => {
					if Markers.is_empty() {
						OwnerMap.remove(&URIKey);
						Vec::new()
					} else {
						let MarkersClone = Markers.clone();
						OwnerMap.insert(URIKey.clone(), Markers);
						MarkersClone
					}
				},
				None => {
					OwnerMap.remove(&URIKey);
					Vec::new()
				},
			};

			ChangedEntries.push(json!({
				"uri": URIKey,
				"markers": MarkersForEvent,
			}));
		}

		drop(DiagnosticsMapGuard);

		// Notify the frontend that diagnostics have changed. Both keys are
		// included for backward compatibility - older listeners read `Uris`
		// (string-array) while the new SkyBridge marker bridge reads
		// `changedURIs` (per-URI marker payload) to push directly into
		// the workbench's `IMarkerService`.
		let EventPayload = json!({
			"Owner": Owner,
			"owner": Owner,
			"Uris": ChangedURIKeys,
			"changedURIs": ChangedEntries,
		});

		// Route through `LogSkyEmit` so the channel + payload size lands
		// in the `[DEV:SKY-EMIT]` histogram alongside SCM / tree-view /
		// terminal emits. Diagnostic emit volume is one of the easiest
		// signals to over- or under-count when triaging "Problems panel
		// shows count but no items"; without LogSkyEmit the channel was
		// invisible.
		if let Err(Error) = LogSkyEmit(&self.ApplicationHandle, SkyEvent::DiagnosticsChanged.AsStr(), EventPayload) {
			dev_log!(
				"extensions",
				"error: [DiagnosticProvider] Failed to emit 'diagnostics_changed': {}",
				Error
			);
		}

		dev_log!(
			"extensions",
			"[DiagnosticProvider] Emitted diagnostics changed for {} URI(s)",
			ChangedURIKeys.len()
		);

		Ok(())
	}

	/// Clears all diagnostics from a specific owner.
	async fn ClearDiagnostics(&self, Owner:String) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[DiagnosticProvider] Clearing all diagnostics for owner: {}",
			Owner
		);

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
			dev_log!(
				"extensions",
				"[DiagnosticProvider] Cleared {} diagnostics across {} URI(s)",
				ClearedCount,
				ChangedURIKeys.len()
			);

			// Clear path - every URI's marker set goes to empty so the
			// SkyBridge listener can wipe them via
			// `IMarkerService.changeOne(owner, uri, [])`.
			let ChangedEntries:Vec<serde_json::Value> =
				ChangedURIKeys.iter().map(|Uri| json!({ "uri": Uri, "markers": [] })).collect();
			let EventPayload = json!({
				"Owner": Owner,
				"owner": Owner,
				"Uris": ChangedURIKeys,
				"changedURIs": ChangedEntries,
			});

			if let Err(Error) = LogSkyEmit(&self.ApplicationHandle, SkyEvent::DiagnosticsChanged.AsStr(), EventPayload)
			{
				dev_log!(
					"extensions",
					"error: [DiagnosticProvider] Failed to emit 'diagnostics_changed' on clear: {}",
					Error
				);
			}
		}

		Ok(())
	}

	/// Retrieves all diagnostics, optionally filtered by a resource URI.
	/// Returns diagnostics aggregated from all owners for the specified
	/// resource(s).
	async fn GetAllDiagnostics(&self, ResourceURIFilterOption:Option<Value>) -> Result<Value, CommonError> {
		dev_log!(
			"extensions",
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

		dev_log!(
			"extensions",
			"[DiagnosticProvider] Returning {} diagnostic collection(s)",
			ResultList.len()
		);

		serde_json::to_value(ResultList).map_err(|Error| CommonError::from(Error))
	}
}
