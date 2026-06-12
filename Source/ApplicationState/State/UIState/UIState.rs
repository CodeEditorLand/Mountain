//! # UIState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages user interface request state including pending UI interactions
//! such as dialogs, prompts, and other synchronous UI requests. Uses
//! oneshot channels for request/response communication.
//!
//! ## ARCHITECTURAL ROLE
//! UIState is part of the **state organization layer**, representing
//! user interface request state organized by request ID.
//!
//! ## KEY COMPONENTS
//! - State: Main struct containing pending UI requests map
//! - Default: Initialization implementation
//! - Helper methods: UI request manipulation utilities
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
//! - Use oneshot channels for request/response
//!
//! ## TODO
//! - [ ] Add UI request validation invariants
//! - [ ] Implement UI request timeout handling
//! - [ ] Add UI request metrics collection

use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use CommonLibrary::Error::CommonError::CommonError;

use crate::dev_log;

/// User interface request state containing pending UI interactions.
#[derive(Clone)]
pub struct State {
	/// Pending user interface request organized by request ID.
	/// Each request has a oneshot sender for sending the response back.
	pub PendingUserInterfaceRequest:
		Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>>>>,
}

impl Default for State {
	fn default() -> Self {
		dev_log!("window", "[UIState] Initializing default UI state...");

		Self { PendingUserInterfaceRequest:Arc::new(Mutex::new(HashMap::new())) }
	}
}

impl State {
	/// Gets all pending user interface request IDs.
	/// Note: Returns only the IDs since oneshot::Sender cannot be cloned.
	pub fn GetPendingRequests(&self) -> Vec<String> {
		self.PendingUserInterfaceRequest.lock().keys().cloned().collect()
	}

	/// Adds a pending user interface request.
	pub fn AddPendingRequest(
		&self,

		id:String,

		sender:tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>,
	) {
		let mut guard = self.PendingUserInterfaceRequest.lock();

		guard.insert(id, sender);

		dev_log!("window", "[UIState] Pending UI request added");
	}

	/// Removes a pending user interface request by its ID.
	pub fn RemovePendingRequest(
		&self,

		id:&str,
	) -> Option<tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>> {
		let mut guard = self.PendingUserInterfaceRequest.lock();

		let sender = guard.remove(id);

		dev_log!("window", "[UIState] Pending UI request removed: {}", id);

		sender
	}

	/// Clears all pending user interface requests.
	pub fn ClearAll(&self) {
		let mut guard = self.PendingUserInterfaceRequest.lock();

		guard.clear();

		dev_log!("window", "[UIState] All pending UI requests cleared");
	}

	/// Gets the count of pending user interface requests.
	pub fn Count(&self) -> usize { self.PendingUserInterfaceRequest.lock().len() }

	/// Checks if a pending user interface request exists.
	pub fn Contains(&self, id:&str) -> bool { self.PendingUserInterfaceRequest.lock().contains_key(id) }
}
