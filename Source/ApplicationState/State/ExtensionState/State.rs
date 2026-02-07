//! # State Module (ExtensionState)
//!
//! ## RESPONSIBILITIES
//! Combines all extension-related state components into a single state struct.
//!
//! ## ARCHITECTURAL ROLE
//! State is the main composite struct that combines all ExtensionState components:
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

use super::{ExtensionRegistry::Registry, ProviderRegistration::Registration, ScannedExtensions::Extensions};
use log::debug;

/// Extension state combining all extension-related components.
#[derive(Clone)]
pub struct State {
	/// Extension registry containing command registry and provider state.
	pub Registry: Registry,

	/// Language provider registration state.
	pub ProviderRegistration: Registration,

	/// Scanned extensions containing discovered extensions.
	pub ScannedExtensions: Extensions,
}

impl Default for State {
	fn default() -> Self {
		debug!("[ExtensionState::State] Initializing default extension state...");

		Self {
			Registry: Default::default(),
			ProviderRegistration: Default::default(),
			ScannedExtensions: Default::default(),
		}
	}
}

impl State {
	/// Gets the next available unique identifier for a provider registration.
	pub fn GetNextProviderHandle(&self) -> u32 { self.Registry.GetNextProviderHandle() }
}
