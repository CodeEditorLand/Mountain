//! # ProviderRegistration Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages language providers registration state including completion,
//! hover, document symbol, and other language feature providers.
//!
//! ## ARCHITECTURAL ROLE
//! ProviderRegistration is part of the **ExtensionState** module, representing
//! language provider registration state.
//!
//! ## KEY COMPONENTS
//! - Registration: Main struct containing language providers map
//! - Default: Initialization implementation
//! - Helper methods: Provider manipulation utilities
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
//! - [ ] Add provider validation invariants
//! - [ ] Implement provider lifecycle events
//! - [ ] Add provider metrics collection

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

/// Language provider registration state.
#[derive(Clone)]
pub struct Registration {
	/// Registered language providers by handle.
	pub LanguageProviders:Arc<StandardMutex<HashMap<u32, ProviderRegistrationDTO>>>,
}

impl Default for Registration {
	fn default() -> Self {
		dev_log!(
			"extensions",
			"[ProviderRegistration] Initializing default provider registration..."
		);

		Self { LanguageProviders:Arc::new(StandardMutex::new(HashMap::new())) }
	}
}

impl Registration {
	/// Gets all registered language providers.
	pub fn GetProviders(&self) -> HashMap<u32, ProviderRegistrationDTO> {
		self.LanguageProviders
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}

	/// Gets a provider by its handle.
	pub fn GetProvider(&self, handle:u32) -> Option<ProviderRegistrationDTO> {
		self.LanguageProviders.lock().ok().and_then(|guard| guard.get(&handle).cloned())
	}

	/// Registers a language provider.
	pub fn RegisterProvider(&self, handle:u32, provider:ProviderRegistrationDTO) {
		if let Ok(mut guard) = self.LanguageProviders.lock() {
			guard.insert(handle, provider);
			dev_log!(
				"extensions",
				"[ProviderRegistration] Provider registered with handle: {}",
				handle
			);
		}
	}

	/// Unregisters a language provider.
	pub fn UnregisterProvider(&self, handle:u32) {
		if let Ok(mut guard) = self.LanguageProviders.lock() {
			guard.remove(&handle);
			dev_log!(
				"extensions",
				"[ProviderRegistration] Provider unregistered with handle: {}",
				handle
			);
		}
	}

	/// Gets all providers for a specific language.
	pub fn GetProvidersForLanguage(&self, language:&str) -> Vec<ProviderRegistrationDTO> {
		self.LanguageProviders
			.lock()
			.ok()
			.map(|guard| guard.values().filter(|p| p.MatchesSelector("", language)).cloned().collect())
			.unwrap_or_default()
	}
}
