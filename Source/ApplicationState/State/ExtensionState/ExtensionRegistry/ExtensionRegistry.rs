//! # ExtensionRegistry Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages extension registry including command registry and provider handle
//! management. Tracks extension scan paths and enabled proposed APIs.
//!
//! ## ARCHITECTURAL ROLE
//! ExtensionRegistry is part of the **ExtensionState** module, representing
//! the command registry and provider handle management.
//!
//! ## KEY COMPONENTS
//! - Registry: Main struct containing command registry and provider state
//! - Default: Initialization implementation
//! - Helper methods: Registry manipulation utilities
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
//! - Use AtomicU32 for unique provider handles
//!
//! ## TODO
//! - [ ] Add command validation invariants
//! - [ ] Implement command discovery events
//! - [ ] Add command metrics collection

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{
		Arc,
		Mutex as StandardMutex,
		atomic::{AtomicU32, Ordering as AtomicOrdering},
	},
};

use tauri::Wry;

use crate::{Environment::CommandProvider::CommandHandler, dev_log};

/// Extension registry containing command registry and provider handle state.
#[derive(Clone)]
pub struct Registry {
	/// Registered CLI commands.
	pub CommandRegistry:Arc<StandardMutex<HashMap<String, CommandHandler<Wry>>>>,

	/// Counter for generating unique provider handles.
	pub NextProviderHandle:Arc<AtomicU32>,

	/// Paths to scan for extensions.
	pub ExtensionScanPaths:Arc<StandardMutex<Vec<PathBuf>>>,

	/// Enabled proposed APIs for extensions.
	pub EnabledProposedAPIs:Arc<StandardMutex<HashMap<String, Vec<String>>>>,
}

impl Default for Registry {
	fn default() -> Self {
		dev_log!("extensions", "[ExtensionRegistry] Initializing default extension registry...");

		Self {
			CommandRegistry:Arc::new(StandardMutex::new(HashMap::new())),

			NextProviderHandle:Arc::new(AtomicU32::new(1)),

			ExtensionScanPaths:Arc::new(StandardMutex::new(Vec::new())),

			EnabledProposedAPIs:Arc::new(StandardMutex::new(HashMap::new())),
		}
	}
}

impl Registry {
	/// Gets the next available unique identifier for a provider registration.
	pub fn GetNextProviderHandle(&self) -> u32 {
		let current = self.NextProviderHandle.load(AtomicOrdering::Relaxed);

		if current > std::u32::MAX / 2 {
			dev_log!(
				"extensions",
				"warn: [ExtensionRegistry] GetNextProviderHandle near overflow ({}); wrap is about to degenerate IDs",
				current
			);
		}

		self.NextProviderHandle.fetch_add(1, AtomicOrdering::Relaxed)
	}

	/// Executes a closure with exclusive access to the command registry,
	/// recovering from poison without silently discarding mutations.
	pub fn WithCommands<F, T>(&self, f:F) -> T
	where
		F: FnOnce(&HashMap<String, CommandHandler<Wry>>) -> T, {
		let guard = self.CommandRegistry.lock().unwrap_or_else(|e| {
			dev_log!("extensions", "warn: [ExtensionRegistry] CommandRegistry poisoned: {}", e);

			e.into_inner()
		});

		f(&*guard)
	}

	/// Gets all registered commands.
	pub fn GetCommands(&self) -> HashMap<String, CommandHandler<Wry>> { self.WithCommands(|commands| commands.clone()) }

	/// Registers a command.
	pub fn RegisterCommand(&self, name:String, handler:CommandHandler<Wry>) {
		let name_for_log = name.clone();

		if let Ok(mut guard) = self.CommandRegistry.lock() {
			guard.insert(name, handler);

			dev_log!("extensions", "[ExtensionRegistry] Command registered: {}", name_for_log);
		}
	}

	/// Unregisters a command.
	pub fn UnregisterCommand(&self, name:&str) {
		if let Ok(mut guard) = self.CommandRegistry.lock() {
			guard.remove(name);

			dev_log!("extensions", "[ExtensionRegistry] Command unregistered: {}", name);
		}
	}

	/// Gets all extension scan paths.
	pub fn GetExtensionScanPaths(&self) -> Vec<PathBuf> {
		self.ExtensionScanPaths
			.lock()
			.unwrap_or_else(|e| {
				dev_log!("extensions", "warn: [ExtensionRegistry] ExtensionScanPaths poisoned: {}", e);

				e.into_inner()
			})
			.clone()
	}

	/// Sets the extension scan paths.
	pub fn SetExtensionScanPaths(&self, paths:Vec<PathBuf>) {
		if let Ok(mut guard) = self.ExtensionScanPaths.lock() {
			*guard = paths;
			dev_log!(
				"extensions",
				"[ExtensionRegistry] Extension scan paths updated ({} paths)",
				guard.len()
			);
		}
	}

	/// Adds an extension scan path.
	pub fn AddExtensionScanPath(&self, path:PathBuf) {
		if let Ok(mut guard) = self.ExtensionScanPaths.lock() {
			guard.push(path.clone());

			dev_log!("extensions", "[ExtensionRegistry] Extension scan path added: {:?}", path);
		}
	}

	/// Gets all enabled proposed APIs.
	pub fn GetEnabledProposedAPIs(&self) -> HashMap<String, Vec<String>> {
		self.EnabledProposedAPIs
			.lock()
			.unwrap_or_else(|e| {
				dev_log!("extensions", "warn: [ExtensionRegistry] EnabledProposedAPIs poisoned: {}", e);

				e.into_inner()
			})
			.clone()
	}

	/// Sets the enabled proposed APIs.
	pub fn SetEnabledProposedAPIs(&self, apis:HashMap<String, Vec<String>>) {
		if let Ok(mut guard) = self.EnabledProposedAPIs.lock() {
			*guard = apis;
			dev_log!(
				"extensions",
				"[ExtensionRegistry] Enabled proposed APIs updated ({} entries)",
				guard.len()
			);
		}
	}

	/// Enables a proposed API for an extension.
	pub fn EnableProposedAPI(&self, extension_id:String, api_name:String) {
		if let Ok(mut guard) = self.EnabledProposedAPIs.lock() {
			guard.entry(extension_id).or_insert_with(Vec::new).push(api_name);

			dev_log!("extensions", "[ExtensionRegistry] Proposed API enabled");
		}
	}
}
