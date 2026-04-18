//! # ScannedExtensions Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages scanned extensions metadata state including extension descriptions,
//! capabilities, and identifiers.
//!
//! ## ARCHITECTURAL ROLE
//! ScannedExtensions is part of the **ExtensionState** module, representing
//! discovered extensions metadata state.
//!
//! ## KEY COMPONENTS
//! - Extensions: Main struct containing scanned extensions map
//! - Default: Initialization implementation
//! - Helper methods: Extension manipulation utilities
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
//! - [ ] Add extension validation invariants
//! - [ ] Implement extension discovery events
//! - [ ] Add extension metrics collection

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

/// Scanned extensions containing discovered extension metadata.
#[derive(Clone)]
pub struct ScannedExtensionCollection {
	/// Scanned extensions by identifier.
	pub ScannedExtensions:Arc<StandardMutex<HashMap<String, ExtensionDescriptionStateDTO>>>,
}

impl Default for ScannedExtensionCollection {
	fn default() -> Self {
		dev_log!("extensions", "[ScannedExtensions] Initializing default scanned extensions...");

		Self { ScannedExtensions:Arc::new(StandardMutex::new(HashMap::new())) }
	}
}

impl ScannedExtensionCollection {
	/// Gets all scanned extensions.
	pub fn GetAll(&self) -> HashMap<String, ExtensionDescriptionStateDTO> {
		self.ScannedExtensions
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}

	/// Gets an extension by its identifier.
	pub fn Get(&self, identifier:&str) -> Option<ExtensionDescriptionStateDTO> {
		self.ScannedExtensions
			.lock()
			.ok()
			.and_then(|guard| guard.get(identifier).cloned())
	}

	/// Sets all scanned extensions.
	pub fn SetAll(&self, extensions:HashMap<String, ExtensionDescriptionStateDTO>) {
		if let Ok(mut guard) = self.ScannedExtensions.lock() {
			*guard = extensions;
			dev_log!(
				"extensions",
				"[ScannedExtensions] Scanned extensions updated ({} extensions)",
				guard.len()
			);
		}
	}

	/// Adds or updates an extension.
	pub fn AddOrUpdate(&self, identifier:String, extension:ExtensionDescriptionStateDTO) {
		if let Ok(mut guard) = self.ScannedExtensions.lock() {
			guard.insert(identifier, extension);
			dev_log!("extensions", "[ScannedExtensions] Extension added/updated");
		}
	}

	/// Removes an extension by its identifier.
	pub fn Remove(&self, identifier:&str) {
		if let Ok(mut guard) = self.ScannedExtensions.lock() {
			guard.remove(identifier);
			dev_log!("extensions", "[ScannedExtensions] Extension removed: {}", identifier);
		}
	}

	/// Clears all scanned extensions.
	pub fn Clear(&self) {
		if let Ok(mut guard) = self.ScannedExtensions.lock() {
			guard.clear();
			dev_log!("extensions", "[ScannedExtensions] All extensions cleared");
		}
	}

	/// Gets the count of scanned extensions.
	pub fn Count(&self) -> usize { self.ScannedExtensions.lock().ok().map(|guard| guard.len()).unwrap_or(0) }

	/// Checks if an extension exists.
	pub fn Contains(&self, identifier:&str) -> bool {
		self.ScannedExtensions
			.lock()
			.ok()
			.map(|guard| guard.contains_key(identifier))
			.unwrap_or(false)
	}
}
