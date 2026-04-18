//! # TerminalState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages terminal instances state including terminal metadata, content, and
//! unique identifier tracking.
//!
//! ## ARCHITECTURAL ROLE
//! TerminalState is part of the **FeatureState** module, representing
//! terminal instances state organized by terminal ID.
//!
//! ## KEY COMPONENTS
//! - TerminalState: Main struct containing active terminals map and counter
//! - Default: Initialization implementation
//! - Helper methods: Terminal manipulation utilities
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
//! - Use double mutex for terminals (outer for map, inner for each terminal)
//!
//! ## TODO
//! - [ ] Add terminal validation invariants
//! - [ ] Implement terminal lifecycle events
//! - [ ] Add terminal metrics collection

use std::{
	collections::HashMap,
	sync::{
		Arc,
		Mutex as StandardMutex,
		atomic::{AtomicU64, Ordering as AtomicOrdering},
	},
};

use crate::{ApplicationState::DTO::TerminalStateDTO::TerminalStateDTO, dev_log};

/// Active terminals state containing terminals by ID with next identifier
/// counter.
#[derive(Clone)]
pub struct TerminalState {
	/// Active terminals organized by ID.
	pub ActiveTerminals:Arc<StandardMutex<HashMap<u64, Arc<StandardMutex<TerminalStateDTO>>>>>,

	/// Counter for generating unique terminal identifiers.
	pub NextTerminalIdentifier:Arc<AtomicU64>,
}

impl Default for TerminalState {
	fn default() -> Self {
		dev_log!("terminal", "[TerminalState] Initializing default terminal state...");

		Self {
			ActiveTerminals:Arc::new(StandardMutex::new(HashMap::new())),
			NextTerminalIdentifier:Arc::new(AtomicU64::new(1)),
		}
	}
}

impl TerminalState {
	/// Gets the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.NextTerminalIdentifier.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Gets all active terminals.
	pub fn GetAll(&self) -> HashMap<u64, TerminalStateDTO> {
		self.ActiveTerminals
			.lock()
			.ok()
			.map(|guard| {
				guard
					.iter()
					.filter_map(|(id, arc)| arc.lock().ok().map(|dto| (*id, dto.clone())))
					.collect()
			})
			.unwrap_or_default()
	}

	/// Gets a terminal by its ID.
	pub fn Get(&self, id:u64) -> Option<TerminalStateDTO> {
		self.ActiveTerminals
			.lock()
			.ok()
			.and_then(|guard| guard.get(&id).and_then(|arc| arc.lock().ok().map(|dto| dto.clone())))
	}

	/// Gets a terminal's Arc<Mutex<>> by its ID for direct manipulation.
	pub fn GetArc(&self, id:u64) -> Option<Arc<StandardMutex<TerminalStateDTO>>> {
		self.ActiveTerminals.lock().ok().and_then(|guard| guard.get(&id).cloned())
	}

	/// Adds or updates a terminal.
	pub fn AddOrUpdate(&self, id:u64, terminal:TerminalStateDTO) {
		if let Ok(mut guard) = self.ActiveTerminals.lock() {
			guard.insert(id, Arc::new(StandardMutex::new(terminal)));
			dev_log!("terminal", "[TerminalState] Terminal added/updated with ID: {}", id);
		}
	}

	/// Removes a terminal by its ID.
	pub fn Remove(&self, id:u64) {
		if let Ok(mut guard) = self.ActiveTerminals.lock() {
			guard.remove(&id);
			dev_log!("terminal", "[TerminalState] Terminal removed with ID: {}", id);
		}
	}

	/// Clears all active terminals.
	pub fn Clear(&self) {
		if let Ok(mut guard) = self.ActiveTerminals.lock() {
			guard.clear();
			dev_log!("terminal", "[TerminalState] All terminals cleared");
		}
	}

	/// Gets the count of active terminals.
	pub fn Count(&self) -> usize { self.ActiveTerminals.lock().ok().map(|guard| guard.len()).unwrap_or(0) }

	/// Checks if a terminal exists.
	pub fn Contains(&self, id:u64) -> bool {
		self.ActiveTerminals
			.lock()
			.ok()
			.map(|guard| guard.contains_key(&id))
			.unwrap_or(false)
	}
}
