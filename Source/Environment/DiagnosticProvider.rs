//! # DiagnosticProvider (Environment)
//!
//! Implements the `DiagnosticManager` trait, managing diagnostic information
//! from multiple sources (language servers, extensions, built-in providers). It
//! aggregates diagnostics by owner, file URI, and severity, and notifies the
//! UI when changes occur.
//!
//! Diagnostics are stored in `ApplicationState.Feature.Diagnostics` as a
//! nested `HashMap<owner, HashMap<uri, Vec<MarkerDataDTO>>>`. Each owner (e.g.
//! `"typescript"`, `"rust-analyzer"`) manages its collection independently.
//!
//! ## Data model
//!
//! Each `MarkerDataDTO` carries:
//! - `Severity` - Error(8), Warning(4), Information(2), Hint(1)
//! - `Message` - human-readable description
//! - `StartLineNumber` / `StartColumn` - 1-based (Cocoon's
//!   `NormaliseDiagnostic` adds `+1`)
//! - `EndLineNumber` / `EndColumn` - 1-based, same convention
//! - `Source` - diagnostic source string (e.g. `"tslint"`)
//! - `Code` - diagnostic code for quick-fix lookup
//! - `ModelVersionId` - document version for change tracking
//!
//! ## Notification flow
//!
//! 1. Language server or extension calls `SetDiagnostics(owner, entries)`.
//! 2. Provider validates and stores in `ApplicationState.Feature.Diagnostics`.
//! 3. Provider identifies which URIs changed in this update.
//! 4. Provider emits `sky://diagnostics/changed` with `owner`, `uris` (string
//!    array for back-compat), and `changedURIs` (per-URI marker payload for the
//!    SkyBridge marker bridge).
//! 5. Sky receives the event and updates squiggles and the Problems panel.
//!
//! ## VS Code reference
//!
//! - `vs/workbench/services/diagnostic/common/diagnosticCollection.ts`
//! - `vs/platform/diagnostics/common/diagnostics.ts`
//! - `vs/workbench/services/diagnostic/common/diagnosticService.ts`

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

// TODO: severity filtering, code actions/quick-fix integration, diagnostic
// inline messages, history/undo-redo, export, suppression comments,
// telemetry, remote diagnostics, caching, workspace-wide filtering.
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
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

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
			let URIKey = match Utility::UriParsing::Fn(&URIComponentsValue) {
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
				.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

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

		// Clone only the data needed for aggregation, then release the lock
		// immediately so concurrent SetDiagnostics/ClearDiagnostics calls
		// are not blocked during the (potentially large) serialize step.
		let FilterURIKey = ResourceURIFilterOption
			.as_ref()
			.map(|V| Utility::UriParsing::Fn(V).map(|U| U.to_string()))
			.transpose()?;

		let Snapshot:Vec<std::collections::HashMap<String, Vec<MarkerDataDTO>>> = {
			let DiagnosticsMapGuard = self
				.ApplicationState
				.Feature
				.Diagnostics
				.DiagnosticsMap
				.lock()
				.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

			DiagnosticsMapGuard.values().cloned().collect()
		};

		let mut ResultMap:std::collections::HashMap<String, Vec<MarkerDataDTO>> = std::collections::HashMap::new();

		if let Some(FilterKey) = FilterURIKey {
			for OwnerMap in &Snapshot {
				if let Some(Markers) = OwnerMap.get(&FilterKey) {
					ResultMap.entry(FilterKey.clone()).or_default().extend(Markers.clone());
				}
			}
		} else {
			for OwnerMap in &Snapshot {
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
