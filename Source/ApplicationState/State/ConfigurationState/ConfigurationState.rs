//! # ConfigurationState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//!
//! Manages configuration and storage state including global configuration,
//! workspace configuration, and memento storage buffers.
//!
//! ## ARCHITECTURAL ROLE
//!
//! ConfigurationState is part of the **state organization layer**, representing
//! all configuration and storage-related state in the application. This
//! includes:
//! - Global configuration (merged from all sources)
//! - Workspace configuration
//! - Global memento storage (for crash recovery)
//! - Workspace memento storage
//!
//! ## KEY COMPONENTS
//! - State: Main struct containing configuration and storage fields
//! - Default: Initialization implementation
//! - Helper methods: Configuration and storage manipulation utilities
//!
//! ## ERROR HANDLING
//!
//! - Thread-safe access via `Arc<Mutex<...>>`
//! - Proper lock error handling with `MapLockError` helpers
//!
//! ## LOGGING
//!
//! State changes are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Lock mutexes briefly and release immediately
//! - Avoid nested locks to prevent deadlocks
//! - Use Arc for shared ownership across threads
//!
//! ## TODO
//! - [ ] Add configuration validation invariants
//! - [ ] Implement configuration change events
//! - [ ] Add configuration diffing

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

/// Configuration and storage state.
#[derive(Clone)]
pub struct State {
	/// Merged global configuration from all sources.
	pub GlobalConfiguration:Arc<StandardMutex<serde_json::Value>>,

	/// Merged workspace configuration from all sources.
	pub WorkspaceConfiguration:Arc<StandardMutex<serde_json::Value>>,

	/// Global memento storage for crash recovery.
	pub MementoGlobalStorage:Arc<StandardMutex<HashMap<String, serde_json::Value>>>,

	/// Workspace memento storage for crash recovery.
	pub MementoWorkspaceStorage:Arc<StandardMutex<HashMap<String, serde_json::Value>>>,
}

impl Default for State {
	fn default() -> Self {
		dev_log!("config", "[ConfigurationState] Initializing default configuration state...");

		Self {
			GlobalConfiguration:Arc::new(StandardMutex::new(serde_json::Value::Object(serde_json::Map::new()))),
			WorkspaceConfiguration:Arc::new(StandardMutex::new(serde_json::Value::Object(serde_json::Map::new()))),
			MementoGlobalStorage:Arc::new(StandardMutex::new(HashMap::new())),
			MementoWorkspaceStorage:Arc::new(StandardMutex::new(HashMap::new())),
		}
	}
}

impl State {
	/// Gets the global configuration.
	pub fn GetGlobalConfiguration(&self) -> serde_json::Value {
		self.GlobalConfiguration
			.lock()
			.map(|g| g.clone())
			.unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
	}

	/// Sets the global configuration.
	pub fn SetGlobalConfiguration(&self, config:serde_json::Value) {
		if let Ok(mut guard) = self.GlobalConfiguration.lock() {
			*guard = config;
			dev_log!("config", "[ConfigurationState] Global configuration updated");
		}
	}

	/// Gets the workspace configuration.
	pub fn GetWorkspaceConfiguration(&self) -> serde_json::Value {
		self.WorkspaceConfiguration
			.lock()
			.map(|g| g.clone())
			.unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
	}

	/// Sets the workspace configuration.
	pub fn SetWorkspaceConfiguration(&self, config:serde_json::Value) {
		if let Ok(mut guard) = self.WorkspaceConfiguration.lock() {
			*guard = config;
			dev_log!("config", "[ConfigurationState] Workspace configuration updated");
		}
	}

	/// Gets a value from global configuration at a specific path.
	pub fn GetGlobalValue(&self, path:&str) -> Option<serde_json::Value> {
		self.GetGlobalConfiguration().get(path).cloned()
	}

	/// Sets a value in global configuration at a specific path.
	/// This implementation uses the MergedConfigurationStateDTO's SetValue
	/// method which properly handles nested object creation and value
	/// assignment.
	pub fn SetGlobalValue(&self, path:&str, value:serde_json::Value) {
		if let Ok(mut config_guard) = self.GlobalConfiguration.lock() {
			// Clone the current config for manipulation
			let current_config = (*config_guard).clone();

			// Create DTO to leverage its SetValue method
			let mut dto = MergedConfigurationStateDTO { Data:current_config };

			// Use the DTO's SetValue method which handles nested paths properly
			if let Err(e) = dto.SetValue(path, value) {
				dev_log!(
					"config",
					"warn: [ConfigurationState] Failed to set value at path '{}': {}",
					path,
					e
				);
				return;
			}

			// Write the updated data back
			*config_guard = dto.Data;

			dev_log!("config", "[ConfigurationState] Global configuration value updated at: {}", path);
		}
	}

	/// Gets all global memento storage.
	pub fn GetGlobalMemento(&self) -> HashMap<String, serde_json::Value> {
		self.MementoGlobalStorage
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}

	/// Sets all global memento storage.
	pub fn SetGlobalMemento(&self, storage:HashMap<String, serde_json::Value>) {
		if let Ok(mut guard) = self.MementoGlobalStorage.lock() {
			*guard = storage;
			dev_log!(
				"config",
				"[ConfigurationState] Global memento storage updated ({} keys)",
				guard.len()
			);
		}
	}

	/// Gets a value from global memento storage.
	pub fn GetGlobalMementoValue(&self, key:&str) -> Option<serde_json::Value> {
		self.MementoGlobalStorage.lock().ok().and_then(|guard| guard.get(key).cloned())
	}

	/// Sets a value in global memento storage.
	pub fn SetGlobalMementoValue(&self, key:String, value:serde_json::Value) {
		if let Ok(mut guard) = self.MementoGlobalStorage.lock() {
			guard.insert(key.clone(), value);
			dev_log!("config", "[ConfigurationState] Global memento value updated for key: {}", key);
		}
	}

	/// Gets all workspace memento storage.
	pub fn GetWorkspaceMemento(&self) -> HashMap<String, serde_json::Value> {
		self.MementoWorkspaceStorage
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}

	/// Sets all workspace memento storage.
	pub fn SetWorkspaceMemento(&self, storage:HashMap<String, serde_json::Value>) {
		if let Ok(mut guard) = self.MementoWorkspaceStorage.lock() {
			*guard = storage;
			dev_log!(
				"config",
				"[ConfigurationState] Workspace memento storage updated ({} keys)",
				guard.len()
			);
		}
	}

	/// Gets a value from workspace memento storage.
	pub fn GetWorkspaceMementoValue(&self, key:&str) -> Option<serde_json::Value> {
		self.MementoWorkspaceStorage
			.lock()
			.ok()
			.and_then(|guard| guard.get(key).cloned())
	}

	/// Sets a value in workspace memento storage.
	pub fn SetWorkspaceMementoValue(&self, key:String, value:serde_json::Value) {
		if let Ok(mut guard) = self.MementoWorkspaceStorage.lock() {
			guard.insert(key.clone(), value);
			dev_log!(
				"config",
				"[ConfigurationState] Workspace memento value updated for key: {}",
				key
			);
		}
	}

	/// Clears a value from workspace memento storage.
	pub fn ClearWorkspaceMementoValue(&self, key:&str) {
		if let Ok(mut guard) = self.MementoWorkspaceStorage.lock() {
			guard.remove(key);
			dev_log!(
				"config",
				"[ConfigurationState] Workspace memento value removed for key: {}",
				key
			);
		}
	}

	/// Clears global memento storage.
	pub fn ClearGlobalMemento(&self) {
		if let Ok(mut guard) = self.MementoGlobalStorage.lock() {
			guard.clear();
			dev_log!("config", "[ConfigurationState] Global memento storage cleared");
		}
	}

	/// Clears workspace memento storage.
	pub fn ClearWorkspaceMemento(&self) {
		if let Ok(mut guard) = self.MementoWorkspaceStorage.lock() {
			guard.clear();
			dev_log!("config", "[ConfigurationState] Workspace memento storage cleared");
		}
	}
}
