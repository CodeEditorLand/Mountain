//! # DocumentState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages open documents state including document metadata, content, and
//! version tracking.
//!
//! ## ARCHITECTURAL ROLE
//! DocumentState is part of the **FeatureState** module, representing
//! open documents state organized by document URI.
//!
//! ## KEY COMPONENTS
//! - DocumentState: Main struct containing open documents map
//! - Default: Initialization implementation
//! - Helper methods: Document manipulation utilities
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
//! - [ ] Add document validation invariants
//! - [ ] Implement document lifecycle events
//! - [ ] Add document metrics collection

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use log::debug;

use crate::ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO;

/// Open documents state containing documents by URI.
#[derive(Clone)]
pub struct DocumentState {
	/// Open documents organized by URI.
	pub OpenDocuments:Arc<StandardMutex<HashMap<String, DocumentStateDTO>>>,
}

impl Default for DocumentState {
	fn default() -> Self {
		debug!("[DocumentState] Initializing default document state...");

		Self { OpenDocuments:Arc::new(StandardMutex::new(HashMap::new())) }
	}
}

impl DocumentState {
	/// Gets all open documents.
	pub fn GetAll(&self) -> HashMap<String, DocumentStateDTO> {
		self.OpenDocuments.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}

	/// Gets a document by its URI.
	pub fn Get(&self, uri:&str) -> Option<DocumentStateDTO> {
		self.OpenDocuments.lock().ok().and_then(|guard| guard.get(uri).cloned())
	}

	/// Adds or updates a document.
	pub fn AddOrUpdate(&self, uri:String, document:DocumentStateDTO) {
		if let Ok(mut guard) = self.OpenDocuments.lock() {
			guard.insert(uri, document);
			debug!("[DocumentState] Document added/updated");
		}
	}

	/// Removes a document by its URI.
	pub fn Remove(&self, uri:&str) {
		if let Ok(mut guard) = self.OpenDocuments.lock() {
			guard.remove(uri);
			debug!("[DocumentState] Document removed: {}", uri);
		}
	}

	/// Clears all open documents.
	pub fn Clear(&self) {
		if let Ok(mut guard) = self.OpenDocuments.lock() {
			guard.clear();
			debug!("[DocumentState] All documents cleared");
		}
	}

	/// Gets the count of open documents.
	pub fn Count(&self) -> usize { self.OpenDocuments.lock().ok().map(|guard| guard.len()).unwrap_or(0) }

	/// Checks if a document exists.
	pub fn Contains(&self, uri:&str) -> bool {
		self.OpenDocuments
			.lock()
			.ok()
			.map(|guard| guard.contains_key(uri))
			.unwrap_or(false)
	}

	/// Gets all document URIs.
	pub fn GetURIs(&self) -> Vec<String> {
		self.OpenDocuments
			.lock()
			.ok()
			.map(|guard| guard.keys().cloned().collect())
			.unwrap_or_default()
	}
}
