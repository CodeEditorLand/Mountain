//! # WebviewState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages webview panels state including webview metadata, content, and
//! presentation state.
//!
//! ## ARCHITECTURAL ROLE
//! WebviewState is part of the **FeatureState** module, representing
//! webview panels state organized by webview ID.
//!
//! ## KEY COMPONENTS
//! - WebviewState: Main struct containing active webviews map
//! - Default: Initialization implementation
//! - Helper methods: Webview manipulation utilities
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
//! - [ ] Add webview validation invariants
//! - [ ] Implement webview lifecycle events
//! - [ ] Add webview metrics collection

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

/// Active webviews state containing webviews by ID.
#[derive(Clone)]
pub struct WebviewState {
	/// Active webviews organized by ID.
	pub ActiveWebviews:Arc<StandardMutex<HashMap<String, WebviewStateDTO>>>,
}

impl Default for WebviewState {
	fn default() -> Self {
		dev_log!("extensions", "[WebviewState] Initializing default webview state...");

		Self { ActiveWebviews:Arc::new(StandardMutex::new(HashMap::new())) }
	}
}

impl WebviewState {
	/// Gets all active webviews.
	pub fn GetAll(&self) -> HashMap<String, WebviewStateDTO> {
		self.ActiveWebviews.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}

	/// Gets a webview by its ID.
	pub fn Get(&self, id:&str) -> Option<WebviewStateDTO> {
		self.ActiveWebviews.lock().ok().and_then(|guard| guard.get(id).cloned())
	}

	/// Adds or updates a webview.
	pub fn AddOrUpdate(&self, id:String, webview:WebviewStateDTO) {
		if let Ok(mut guard) = self.ActiveWebviews.lock() {
			guard.insert(id, webview);
			dev_log!("extensions", "[WebviewState] Webview added/updated");
		}
	}

	/// Removes a webview by its ID.
	pub fn Remove(&self, id:&str) {
		if let Ok(mut guard) = self.ActiveWebviews.lock() {
			guard.remove(id);
			dev_log!("extensions", "[WebviewState] Webview removed: {}", id);
		}
	}

	/// Clears all active webviews.
	pub fn Clear(&self) {
		if let Ok(mut guard) = self.ActiveWebviews.lock() {
			guard.clear();
			dev_log!("extensions", "[WebviewState] All webviews cleared");
		}
	}

	/// Gets the count of active webviews.
	pub fn Count(&self) -> usize { self.ActiveWebviews.lock().ok().map(|guard| guard.len()).unwrap_or(0) }

	/// Checks if a webview exists.
	pub fn Contains(&self, id:&str) -> bool {
		self.ActiveWebviews
			.lock()
			.ok()
			.map(|guard| guard.contains_key(id))
			.unwrap_or(false)
	}
}
