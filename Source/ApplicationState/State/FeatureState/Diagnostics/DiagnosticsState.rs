//! # DiagnosticsState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages diagnostic errors state including markers organized by owner and
//! resource URI. Supports multiple diagnostic owners with their respective
//! marker collections.
//!
//! ## ARCHITECTURAL ROLE
//! DiagnosticsState is part of the **FeatureState** module, representing
//! diagnostic errors state.
//!
//! ## KEY COMPONENTS
//! - DiagnosticsState: Main struct containing diagnostics map
//! - Default: Initialization implementation
//! - Helper methods: Diagnostics manipulation utilities
//!
//! ## ERROR HANDLING
//! - Thread-safe access via `Arc<Mutex<...>>`
//! - Proper lock error handling with `MapLockError` helpers
//!
//! ## LOGGING
//! State changes are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! - Avoid nested locks to prevent deadlocks
//! - Use Arc for shared ownership across threads
//!
//! ## TODO
//! - [ ] Add diagnostics validation invariants
//! - [ ] Implement diagnostics change events
//! - [ ] Add diagnostics metrics collection

use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

/// Diagnostic errors state containing markers by owner and resource.
#[derive(Clone)]
pub struct DiagnosticsState {
	/// Diagnostics map organized by owner and resource URI.
	///
	/// Structure: owner -> resource URI -> list of markers
	pub DiagnosticsMap:Arc<Mutex<HashMap<String, HashMap<String, Vec<MarkerDataDTO>>>>>,
}

impl Default for DiagnosticsState {
	fn default() -> Self {
		dev_log!("extensions", "[DiagnosticsState] Initializing default diagnostics state...");

		Self { DiagnosticsMap:Arc::new(Mutex::new(HashMap::new())) }
	}
}

impl DiagnosticsState {
	/// Gets all diagnostics for all owners and resources.
	pub fn GetAll(&self) -> HashMap<String, HashMap<String, Vec<MarkerDataDTO>>> { self.DiagnosticsMap.lock().clone() }

	/// Gets all diagnostics for a specific owner.
	pub fn GetByOwner(&self, owner:&str) -> HashMap<String, Vec<MarkerDataDTO>> {
		self.DiagnosticsMap.lock().get(owner).cloned().unwrap_or_default()
	}

	/// Gets all diagnostics for a specific owner and resource.
	pub fn GetByOwnerAndResource(&self, owner:&str, resource:&str) -> Vec<MarkerDataDTO> {
		self.DiagnosticsMap
			.lock()
			.get(owner)
			.and_then(|resources| resources.get(resource).cloned())
			.unwrap_or_default()
	}

	/// Sets all diagnostics for a specific owner.
	pub fn SetByOwner(&self, owner:String, diagnostics:HashMap<String, Vec<MarkerDataDTO>>) {
		let mut guard = self.DiagnosticsMap.lock();

		guard.insert(owner, diagnostics);

		dev_log!("extensions", "[DiagnosticsState] Diagnostics updated for owner");
	}

	/// Sets diagnostics for a specific owner and resource.
	pub fn SetByOwnerAndResource(&self, owner:String, resource:String, markers:Vec<MarkerDataDTO>) {
		let mut guard = self.DiagnosticsMap.lock();

		guard.entry(owner).or_insert_with(HashMap::new).insert(resource, markers);

		dev_log!("extensions", "[DiagnosticsState] Diagnostics updated for owner and resource");
	}

	/// Clears all diagnostics for a specific owner.
	pub fn ClearByOwner(&self, owner:&str) {
		let mut guard = self.DiagnosticsMap.lock();

		guard.remove(owner);

		dev_log!("extensions", "[DiagnosticsState] Diagnostics cleared for owner: {}", owner);
	}

	/// Clears diagnostics for a specific owner and resource.
	pub fn ClearByOwnerAndResource(&self, owner:&str, resource:&str) {
		let mut guard = self.DiagnosticsMap.lock();

		if let Some(resources) = guard.get_mut(owner) {
			resources.remove(resource);

			dev_log!("extensions", "[DiagnosticsState] Diagnostics cleared for owner and resource");
		}
	}

	/// Clears all diagnostics.
	pub fn ClearAll(&self) {
		let mut guard = self.DiagnosticsMap.lock();

		guard.clear();

		dev_log!("extensions", "[DiagnosticsState] All diagnostics cleared");
	}
}
