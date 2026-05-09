//! # TreeViewState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages tree view providers state including tree view metadata, data,
//! and presentation state.
//!
//! ## ARCHITECTURAL ROLE
//! TreeViewState is part of the **FeatureState** module, representing
//! tree view providers state organized by tree view ID.
//!
//! ## KEY COMPONENTS
//! - TreeViewState: Main struct containing active tree views map
//! - Default: Initialization implementation
//! - Helper methods: Tree view manipulation utilities
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
//! - [ ] Add tree view validation invariants
//! - [ ] Implement tree view lifecycle events
//! - [ ] Add tree view metrics collection

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use crate::{ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO, dev_log};

/// Active tree views state containing tree views by ID.
#[derive(Clone)]
pub struct TreeViewState {
	/// Active tree views organized by ID.
	pub ActiveTreeViews:Arc<StandardMutex<HashMap<String, TreeViewStateDTO>>>,
}

impl Default for TreeViewState {
	fn default() -> Self {
		dev_log!("extensions", "[TreeViewState] Initializing default tree view state...");

		Self { ActiveTreeViews:Arc::new(StandardMutex::new(HashMap::new())) }
	}
}

impl TreeViewState {
	/// Gets all active tree views.
	pub fn GetAll(&self) -> HashMap<String, TreeViewStateDTO> {
		self.ActiveTreeViews.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}

	/// Gets a tree view by its ID.
	pub fn Get(&self, id:&str) -> Option<TreeViewStateDTO> {
		self.ActiveTreeViews.lock().ok().and_then(|guard| guard.get(id).cloned())
	}

	/// Adds or updates a tree view.
	pub fn AddOrUpdate(&self, id:String, tree_view:TreeViewStateDTO) {
		if let Ok(mut guard) = self.ActiveTreeViews.lock() {
			guard.insert(id, tree_view);

			dev_log!("extensions", "[TreeViewState] Tree view added/updated");
		}
	}

	/// Removes a tree view by its ID.
	pub fn Remove(&self, id:&str) {
		if let Ok(mut guard) = self.ActiveTreeViews.lock() {
			guard.remove(id);

			dev_log!("extensions", "[TreeViewState] Tree view removed: {}", id);
		}
	}

	/// Clears all active tree views.
	pub fn Clear(&self) {
		if let Ok(mut guard) = self.ActiveTreeViews.lock() {
			guard.clear();

			dev_log!("extensions", "[TreeViewState] All tree views cleared");
		}
	}

	/// Gets the count of active tree views.
	pub fn Count(&self) -> usize { self.ActiveTreeViews.lock().ok().map(|guard| guard.len()).unwrap_or(0) }

	/// Checks if a tree view exists.
	pub fn Contains(&self, id:&str) -> bool {
		self.ActiveTreeViews
			.lock()
			.ok()
			.map(|guard| guard.contains_key(id))
			.unwrap_or(false)
	}
}
