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
		atomic::{AtomicU64, Ordering as AtomicOrdering},
	},
};

use parking_lot::Mutex;

use crate::{ApplicationState::DTO::TerminalStateDTO::TerminalStateDTO, dev_log};

/// Active terminals state containing terminals by ID with next identifier
/// counter.
#[derive(Clone)]
pub struct TerminalState {
	/// Active terminals organized by ID.
	pub ActiveTerminals:Arc<Mutex<HashMap<u64, Arc<Mutex<TerminalStateDTO>>>>>,

	/// Counter for generating unique terminal identifiers.
	pub NextTerminalIdentifier:Arc<AtomicU64>,
}

impl Default for TerminalState {
	fn default() -> Self {
		dev_log!("terminal", "[TerminalState] Initializing default terminal state...");

		Self {
			ActiveTerminals:Arc::new(Mutex::new(HashMap::new())),

			NextTerminalIdentifier:Arc::new(AtomicU64::new(1)),
		}
	}
}

impl TerminalState {
	/// Gets the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.NextTerminalIdentifier.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Gets all active terminals.
	pub fn GetAll(&self) -> HashMap<u64, TerminalStateDTO> {
		let guard = self.ActiveTerminals.lock();

		guard
			.iter()
			.filter_map(|(id, arc)| {
				let dto_guard = arc.lock();

				Some((*id, (*dto_guard).clone()))
			})
			.collect()
	}

	/// Gets a terminal by its ID.
	pub fn Get(&self, id:u64) -> Option<TerminalStateDTO> {
		let guard = self.ActiveTerminals.lock();

		guard.get(&id).and_then(|arc| {
			let dto_guard = arc.lock();

			Some((*dto_guard).clone())
		})
	}

	/// Gets a terminal's Arc<Mutex<>> by its ID for direct manipulation.
	pub fn GetArc(&self, id:u64) -> Option<Arc<Mutex<TerminalStateDTO>>> {
		let guard = self.ActiveTerminals.lock();

		guard.get(&id).cloned()
	}

	/// Adds or updates a terminal.
	pub fn AddOrUpdate(&self, id:u64, terminal:TerminalStateDTO) {
		let mut guard = self.ActiveTerminals.lock();

		guard.insert(id, Arc::new(Mutex::new(terminal)));

		dev_log!("terminal", "[TerminalState] Terminal added/updated with ID: {}", id);
	}

	/// Removes a terminal by its ID.
	pub fn Remove(&self, id:u64) {
		let mut guard = self.ActiveTerminals.lock();

		guard.remove(&id);

		dev_log!("terminal", "[TerminalState] Terminal removed with ID: {}", id);
	}

	/// Clears all active terminals.
	pub fn Clear(&self) {
		let mut guard = self.ActiveTerminals.lock();

		guard.clear();

		dev_log!("terminal", "[TerminalState] All terminals cleared");
	}

	/// Gets the count of active terminals.
	pub fn Count(&self) -> usize {
		let guard = self.ActiveTerminals.lock();

		guard.len()
	}

	/// Checks if a terminal exists.
	pub fn Contains(&self, id:u64) -> bool {
		let guard = self.ActiveTerminals.lock();

		guard.contains_key(&id)
	}
}
