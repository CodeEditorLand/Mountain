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

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use log::debug;

use crate::ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO;

/// Diagnostic errors state containing markers by owner and resource.
#[derive(Clone)]
pub struct DiagnosticsState {
	/// Diagnostics map organized by owner and resource URI.
	///
	/// Structure: owner -> resource URI -> list of markers
	pub DiagnosticsMap:Arc<StandardMutex<HashMap<String, HashMap<String, Vec<MarkerDataDTO>>>>>,
}

impl Default for DiagnosticsState {
	fn default() -> Self {
		debug!("[DiagnosticsState] Initializing default diagnostics state...");

		Self { DiagnosticsMap:Arc::new(StandardMutex::new(HashMap::new())) }
	}
}

impl DiagnosticsState {
	/// Gets all diagnostics for all owners and resources.
	pub fn GetAll(&self) -> HashMap<String, HashMap<String, Vec<MarkerDataDTO>>> {
		self.DiagnosticsMap.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}

	/// Gets all diagnostics for a specific owner.
	pub fn GetByOwner(&self, owner:&str) -> HashMap<String, Vec<MarkerDataDTO>> {
		self.DiagnosticsMap
			.lock()
			.ok()
			.and_then(|guard| guard.get(owner).cloned())
			.unwrap_or_default()
	}

	/// Gets all diagnostics for a specific owner and resource.
	pub fn GetByOwnerAndResource(&self, owner:&str, resource:&str) -> Vec<MarkerDataDTO> {
		self.DiagnosticsMap
			.lock()
			.ok()
			.and_then(|guard| guard.get(owner).and_then(|resources| resources.get(resource).cloned()))
			.unwrap_or_default()
	}

	/// Sets all diagnostics for a specific owner.
	pub fn SetByOwner(&self, owner:String, diagnostics:HashMap<String, Vec<MarkerDataDTO>>) {
		if let Ok(mut guard) = self.DiagnosticsMap.lock() {
			guard.insert(owner, diagnostics);
			debug!("[DiagnosticsState] Diagnostics updated for owner");
		}
	}

	/// Sets diagnostics for a specific owner and resource.
	pub fn SetByOwnerAndResource(&self, owner:String, resource:String, markers:Vec<MarkerDataDTO>) {
		if let Ok(mut guard) = self.DiagnosticsMap.lock() {
			guard.entry(owner).or_insert_with(HashMap::new).insert(resource, markers);
			debug!("[DiagnosticsState] Diagnostics updated for owner and resource");
		}
	}

	/// Clears all diagnostics for a specific owner.
	pub fn ClearByOwner(&self, owner:&str) {
		if let Ok(mut guard) = self.DiagnosticsMap.lock() {
			guard.remove(owner);
			debug!("[DiagnosticsState] Diagnostics cleared for owner: {}", owner);
		}
	}

	/// Clears diagnostics for a specific owner and resource.
	pub fn ClearByOwnerAndResource(&self, owner:&str, resource:&str) {
		if let Ok(mut guard) = self.DiagnosticsMap.lock() {
			if let Some(resources) = guard.get_mut(owner) {
				resources.remove(resource);
				debug!("[DiagnosticsState] Diagnostics cleared for owner and resource");
			}
		}
	}

	/// Clears all diagnostics.
	pub fn ClearAll(&self) {
		if let Ok(mut guard) = self.DiagnosticsMap.lock() {
			guard.clear();
			debug!("[DiagnosticsState] All diagnostics cleared");
		}
	}
}
