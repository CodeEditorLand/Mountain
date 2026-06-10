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

/// Auto-reply rule stored by `localPty:installAutoReply`.
///
/// When the PTY reader task encounters a line containing `Match`, it
/// immediately writes `Answer` back to the PTY input channel. This is
/// used by VS Code's shell-integration layer for pseudo-tty prompts
/// (e.g. sudo password prompts that fire before the shell prompt is
/// restored). `UseCustomAnswer` is advisory metadata forwarded from
/// the workbench; Mountain always uses `Answer`.
#[derive(Clone, Debug)]
pub struct AutoReplyRule {
	/// Text fragment to match against PTY output lines.
	pub Match:String,

	/// Response text to write back when `Match` is found.
	pub Answer:String,

	/// Whether the answer was provided by the user (vs. a default).
	pub UseCustomAnswer:bool,
}

/// Active terminals state containing terminals by ID with next identifier
/// counter.
#[derive(Clone)]
pub struct TerminalState {
	/// Active terminals organized by ID.
	pub ActiveTerminals:Arc<Mutex<HashMap<u64, Arc<Mutex<TerminalStateDTO>>>>>,

	/// Counter for generating unique terminal identifiers.
	pub NextTerminalIdentifier:Arc<AtomicU64>,

	/// Map from old (pre-reload) terminal ID to newly assigned ID.
	///
	/// Populated by `localPty:reviveTerminalProcesses`; consumed by
	/// `localPty:getRevivedPtyNewId`. Each entry is popped on first
	/// read so the map stays small across reloads.
	pub RevivedIdMap:Arc<Mutex<HashMap<u64, u64>>>,

	/// Auto-reply rules installed via `localPty:installAutoReply`.
	///
	/// Shared across all terminals for the session. The PTY output
	/// reader in `TerminalProvider` checks every output chunk against
	/// these rules and writes the answer back when a match is found.
	pub AutoReplies:Arc<Mutex<Vec<AutoReplyRule>>>,
}

impl Default for TerminalState {
	fn default() -> Self {
		dev_log!("terminal", "[TerminalState] Initializing default terminal state...");

		Self {
			ActiveTerminals:Arc::new(Mutex::new(HashMap::new())),

			NextTerminalIdentifier:Arc::new(AtomicU64::new(1)),

			RevivedIdMap:Arc::new(Mutex::new(HashMap::new())),

			AutoReplies:Arc::new(Mutex::new(Vec::new())),
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
