//! # State Module (ExtensionState)
//!
//! ## RESPONSIBILITIES
//! Combines all extension-related state components into a single state struct.
//!
//! ## ARCHITECTURAL ROLE
//! State is the main composite struct that combines all ExtensionState
//! components:
//! - ExtensionRegistry: Command registry and provider handle management
//! - ProviderRegistration: Language providers registration
//! - ScannedExtensions: Discovered extensions metadata
//!
//! ## KEY COMPONENTS
//! - State: Main struct combining all extension state
//! - Default: Initialization implementation
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
//! - [ ] Add extension state validation invariants
//! - [ ] Implement extension lifecycle events
//! - [ ] Add extension state metrics collection

use std::sync::Arc;

use tokio::sync::Notify;

use super::{ExtensionRegistry, ProviderRegistration, ScannedExtensions};
use crate::dev_log;

/// Extension state combining all extension-related components.
#[derive(Clone)]
pub struct State {
	/// Extension registry containing command registry and provider state.
	pub Registry:ExtensionRegistry::ExtensionRegistry::Registry,

	/// Language provider registration state.
	pub ProviderRegistration:ProviderRegistration::ProviderRegistration::Registration,

	/// Scanned extensions containing discovered extensions.
	pub ScannedExtensions:ScannedExtensions::ScannedExtensions::ScannedExtensionCollection,

	/// Fires once when the initial extension scan has written at least one
	/// extension into `ScannedExtensions`. Callers that need extensions
	/// on the first request (e.g. `extensions:getInstalled` during boot)
	/// can `await` this instead of polling.
	pub ScanReady:Arc<Notify>,
}

impl Default for State {
	fn default() -> Self {
		dev_log!("extensions", "[ExtensionState::State] Initializing default extension state...");

		Self {
			Registry:ExtensionRegistry::ExtensionRegistry::Registry::default(),

			ProviderRegistration:ProviderRegistration::ProviderRegistration::Registration::default(),

			ScannedExtensions:ScannedExtensions::ScannedExtensions::ScannedExtensionCollection::default(),

			ScanReady:Arc::new(Notify::new()),
		}
	}
}

impl State {
	/// Gets the next available unique identifier for a provider registration.
	pub fn GetNextProviderHandle(&self) -> u32 { self.Registry.GetNextProviderHandle() }
}
