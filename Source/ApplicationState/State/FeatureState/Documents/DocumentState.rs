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

use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

/// Open documents state containing documents by URI.
#[derive(Clone)]
pub struct DocumentState {
	/// Open documents organized by URI.
	pub OpenDocuments:Arc<Mutex<HashMap<String, DocumentStateDTO>>>,
}

impl Default for DocumentState {
	fn default() -> Self {
		dev_log!("model", "[DocumentState] Initializing default document state...");

		Self { OpenDocuments:Arc::new(Mutex::new(HashMap::new())) }
	}
}

impl DocumentState {
	/// Gets all open documents.
	pub fn GetAll(&self) -> HashMap<String, DocumentStateDTO> { self.OpenDocuments.lock().clone() }

	/// Gets a document by its URI.
	pub fn Get(&self, uri:&str) -> Option<DocumentStateDTO> { self.OpenDocuments.lock().get(uri).cloned() }

	/// Adds or updates a document.
	pub fn AddOrUpdate(&self, uri:String, document:DocumentStateDTO) {
		let mut guard = self.OpenDocuments.lock();

		guard.insert(uri, document);

		dev_log!("model", "[DocumentState] Document added/updated");
	}

	/// Removes a document by its URI.
	pub fn Remove(&self, uri:&str) {
		let mut guard = self.OpenDocuments.lock();

		guard.remove(uri);

		dev_log!("model", "[DocumentState] Document removed: {}", uri);
	}

	/// Clears all open documents.
	pub fn Clear(&self) {
		let mut guard = self.OpenDocuments.lock();

		guard.clear();

		dev_log!("model", "[DocumentState] All documents cleared");
	}

	/// Gets the count of open documents.
	pub fn Count(&self) -> usize { self.OpenDocuments.lock().len() }

	/// Checks if a document exists.
	pub fn Contains(&self, uri:&str) -> bool { self.OpenDocuments.lock().contains_key(uri) }

	/// Gets all document URIs.
	pub fn GetURIs(&self) -> Vec<String> { self.OpenDocuments.lock().keys().cloned().collect() }
}
