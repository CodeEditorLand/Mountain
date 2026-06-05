//! # MarkerState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages marker-related state including custom documents, status bar items,
//! and source control management (SCM) state.
//!
//! ## ARCHITECTURAL ROLE
//! MarkerState is part of the **FeatureState** module, representing
//! marker-related state including:
//! - Custom documents
//! - Status bar items
//! - SCM providers, groups, and resources
//! - SCM provider handle counter
//!
//! ## KEY COMPONENTS
//! - MarkerState: Main struct containing marker-related state and counter
//! - Default: Initialization implementation
//! - Helper methods: Marker manipulation utilities
//!
//! ## ERROR HANDLING
//! - Thread-safe access via `Arc<Mutex<...>>`
//!
//! ## LOGGING
//! State changes are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! - Avoid nested locks to prevent deadlocks
//! - Use Arc for shared ownership across threads
//! - Use AtomicU32 for unique SCM provider handles
//!
//! ## TODO
//! - [ ] Add marker validation invariants
//! - [ ] Implement marker lifecycle events
//! - [ ] Add marker metrics collection

use std::{
	collections::HashMap,
	sync::{
		Arc,
		atomic::{AtomicU32, Ordering as AtomicOrdering},
	},
};

use parking_lot::Mutex;

use CommonLibrary::{
	SourceControlManagement::DTO::{
		SourceControlManagementGroupDTO::SourceControlManagementGroupDTO,
		SourceControlManagementProviderDTO::SourceControlManagementProviderDTO,
		SourceControlManagementResourceDTO::SourceControlManagementResourceDTO,
	},
	StatusBar::DTO::StatusBarEntryDTO::StatusBarEntryDTO,
};

use crate::{ApplicationState::DTO::CustomDocumentStateDTO::CustomDocumentStateDTO, dev_log};

/// Marker-related state containing custom documents, status bar, and SCM state.
#[derive(Clone)]
pub struct MarkerState {
	/// Active custom documents organized by ID.
	pub ActiveCustomDocuments:Arc<Mutex<HashMap<String, CustomDocumentStateDTO>>>,

	/// Active status bar items organized by ID.
	pub ActiveStatusBarItems:Arc<Mutex<HashMap<String, StatusBarEntryDTO>>>,

	/// SCM providers organized by handle.
	pub SourceControlManagementProviders:Arc<Mutex<HashMap<u32, SourceControlManagementProviderDTO>>>,

	/// SCM groups organized by provider handle and group ID.
	pub SourceControlManagementGroups:
		Arc<Mutex<HashMap<u32, HashMap<String, SourceControlManagementGroupDTO>>>>,

	/// SCM resources organized by provider handle and group ID.
	pub SourceControlManagementResources:
		Arc<Mutex<HashMap<u32, HashMap<String, Vec<SourceControlManagementResourceDTO>>>>>,

	/// Counter for generating unique SCM provider handles.
	pub NextSourceControlManagementProviderHandle:Arc<AtomicU32>,
}

impl Default for MarkerState {
	fn default() -> Self {
		dev_log!("extensions", "[MarkerState] Initializing default marker state...");

		Self {
			ActiveCustomDocuments:Arc::new(Mutex::new(HashMap::new())),

			ActiveStatusBarItems:Arc::new(Mutex::new(HashMap::new())),

			SourceControlManagementProviders:Arc::new(Mutex::new(HashMap::new())),

			SourceControlManagementGroups:Arc::new(Mutex::new(HashMap::new())),

			SourceControlManagementResources:Arc::new(Mutex::new(HashMap::new())),

			NextSourceControlManagementProviderHandle:Arc::new(AtomicU32::new(1)),
		}
	}
}

impl MarkerState {
	/// Gets the next available unique identifier for an SCM provider.
	pub fn GetNextSourceControlManagementProviderHandle(&self) -> u32 {
		self.NextSourceControlManagementProviderHandle
			.fetch_add(1, AtomicOrdering::Relaxed)
	}

	/// Gets all active custom documents.
	pub fn GetCustomDocuments(&self) -> HashMap<String, CustomDocumentStateDTO> {
		self.ActiveCustomDocuments.lock().clone()
	}

	/// Adds or updates a custom document.
	pub fn AddOrUpdateCustomDocument(&self, id:String, document:CustomDocumentStateDTO) {
		let mut guard = self.ActiveCustomDocuments.lock();
		guard.insert(id, document);

		dev_log!("extensions", "[MarkerState] Custom document added/updated");
	}

	/// Removes a custom document by its ID.
	pub fn RemoveCustomDocument(&self, id:&str) {
		let mut guard = self.ActiveCustomDocuments.lock();
		guard.remove(id);

		dev_log!("extensions", "[MarkerState] Custom document removed: {}", id);
	}

	/// Gets all active status bar items.
	pub fn GetStatusBarItems(&self) -> HashMap<String, StatusBarEntryDTO> {
		self.ActiveStatusBarItems.lock().clone()
	}

	/// Adds or updates a status bar item.
	pub fn AddOrUpdateStatusBarItem(&self, id:String, item:StatusBarEntryDTO) {
		let mut guard = self.ActiveStatusBarItems.lock();
		guard.insert(id, item);

		dev_log!("extensions", "[MarkerState] Status bar item added/updated");
	}

	/// Removes a status bar item by its ID.
	pub fn RemoveStatusBarItem(&self, id:&str) {
		let mut guard = self.ActiveStatusBarItems.lock();
		guard.remove(id);

		dev_log!("extensions", "[MarkerState] Status bar item removed: {}", id);
	}
}