//! # DebugState Module (ApplicationState)

//! ## RESPONSIBILITIES
//! Manages debug provider state including debug configuration providers and
//! adapter descriptor factories.

//! ## ARCHITECTURAL ROLE
//! DebugState is part of the **FeatureState** module, storing debug provider
//! registrations keyed by debug type.

//! ## KEY COMPONENTS
//! - DebugState: Main struct containing debug provider registrations
//! - Default: Initialization implementation
//! - Helper methods: Debug registration management

//! ## ERROR HANDLING
//! - Thread-safe access via `Arc<tokio::sync::RwLock<...>>`
//! - Proper lock error handling

//! ## LOGGING
//! State changes are logged at appropriate levels (debug, info, warn, error).

//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! - Use Arc for shared ownership across threads

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use crate::dev_log;

/// Debug configuration provider registration info
#[derive(Clone, Debug)]
pub struct DebugConfigurationProviderRegistration {
	/// The provider handle
	pub ProviderHandle:u32,
	/// The sidecar identifier hosting this provider
	pub SideCarIdentifier:String,
}

/// Debug adapter descriptor factory registration info
#[derive(Clone, Debug)]
pub struct DebugAdapterDescriptorFactoryRegistration {
	/// The factory handle
	pub FactoryHandle:u32,
	/// The sidecar identifier hosting this factory
	pub SideCarIdentifier:String,
}

/// Debug state containing debug provider registrations.
#[derive(Clone)]
pub struct DebugState {
	/// Debug configuration providers organized by debug type.
	pub DebugConfigurationProviders:Arc<StandardMutex<HashMap<String, DebugConfigurationProviderRegistration>>>,
	/// Debug adapter descriptor factories organized by debug type.
	pub DebugAdapterDescriptorFactories:Arc<StandardMutex<HashMap<String, DebugAdapterDescriptorFactoryRegistration>>>,
}

impl Default for DebugState {
	fn default() -> Self {
		dev_log!("exthost", "[DebugState] Initializing default debug state...");

		Self {
			DebugConfigurationProviders:Arc::new(StandardMutex::new(HashMap::new())),
			DebugAdapterDescriptorFactories:Arc::new(StandardMutex::new(HashMap::new())),
		}
	}
}

impl DebugState {
	/// Registers a debug configuration provider.
	pub fn RegisterDebugConfigurationProvider(
		&self,
		debug_type:String,
		provider_handle:u32,
		sidecar_identifier:String,
	) -> Result<(), String> {
		let mut guard = self
			.DebugConfigurationProviders
			.lock()
			.map_err(|e| format!("Failed to lock debug configuration providers: {}", e))?;

		guard.insert(
			debug_type,
			DebugConfigurationProviderRegistration {
				ProviderHandle:provider_handle,
				SideCarIdentifier:sidecar_identifier,
			},
		);

		Ok(())
	}

	/// Gets a debug configuration provider registration by debug type.
	pub fn GetDebugConfigurationProvider(&self, debug_type:&str) -> Option<DebugConfigurationProviderRegistration> {
		self.DebugConfigurationProviders
			.lock()
			.ok()
			.and_then(|guard| guard.get(debug_type).cloned())
	}

	/// Registers a debug adapter descriptor factory.
	pub fn RegisterDebugAdapterDescriptorFactory(
		&self,
		debug_type:String,
		factory_handle:u32,
		sidecar_identifier:String,
	) -> Result<(), String> {
		let mut guard = self
			.DebugAdapterDescriptorFactories
			.lock()
			.map_err(|e| format!("Failed to lock debug adapter descriptor factories: {}", e))?;

		guard.insert(
			debug_type,
			DebugAdapterDescriptorFactoryRegistration {
				FactoryHandle:factory_handle,
				SideCarIdentifier:sidecar_identifier,
			},
		);

		Ok(())
	}

	/// Gets a debug adapter descriptor factory registration by debug type.
	pub fn GetDebugAdapterDescriptorFactory(
		&self,
		debug_type:&str,
	) -> Option<DebugAdapterDescriptorFactoryRegistration> {
		self.DebugAdapterDescriptorFactories
			.lock()
			.ok()
			.and_then(|guard| guard.get(debug_type).cloned())
	}

	/// Gets all registered debug configuration providers.
	pub fn GetAllDebugConfigurationProviders(&self) -> HashMap<String, DebugConfigurationProviderRegistration> {
		self.DebugConfigurationProviders
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}

	/// Gets all registered debug adapter descriptor factories.
	pub fn GetAllDebugAdapterDescriptorFactories(&self) -> HashMap<String, DebugAdapterDescriptorFactoryRegistration> {
		self.DebugAdapterDescriptorFactories
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}
}
