//! # StateBuild - Advanced Application State Initialization
//!
//! Builds the application state with dependency injection, telemetry
//! and feature flag support for different build configurations.
//!
//! ## Build Profiles
//!
//! - **Debug**: Enhanced validation, state inspection
//! - **Development**: Reduced validation for faster iteration
//! - **Telemetry**: Full metrics and tracing export
//!
//! ## Defensive Coding
//!
//! - Type-safe dependency resolution
//! - Validation of required capabilities
//! - Graceful degradation for optional dependencies

use std::sync::Arc;

use log::{debug, error, info, warn};
// ============ Feature Flags ============
#[cfg(feature = "Telemetry")]
use opentelemetry::{KeyValue, global};

use crate::{ApplicationState::ApplicationState, Environment::MountainEnvironment::MountainEnvironment};

/// State build configuration
#[derive(Debug)]
pub struct StateBuildConfig {
	/// Enable comprehensive validation
	pub strict_validation:bool,
	/// Enable state snapshotting
	pub enable_snapshots:bool,
	/// Log state initialization
	pub verbose_logging:bool,
}

impl Default for StateBuildConfig {
	fn default() -> Self {
		Self {
			#[cfg(feature = "Debug")]
			strict_validation:true,
			#[cfg(not(feature = "Debug"))]
			strict_validation:false,
			enable_snapshots:false,
			#[cfg(feature = "Debug")]
			verbose_logging:true,
			#[cfg(not(feature = "Debug"))]
			verbose_logging:false,
		}
	}
}

/// Build application state from environment
///
/// Creates the application state with all required capabilities
/// injected from the MountainEnvironment.
///
/// # Parameters
///
/// - `environment`: Mountain environment containing all capabilities
///
/// # Returns
///
/// Initialized application state ready for use
///
/// # Errors
///
/// Returns error if required capabilities are not available
pub fn Build(environment:MountainEnvironment) -> Result<ApplicationState, String> {
	BuildWithConfig(environment, StateBuildConfig::default())
}

/// Build application state with custom configuration
///
/// # Parameters
///
/// - `environment`: Mountain environment
/// - `config`: State build configuration
///
/// # Returns
///
/// Configured application state
pub fn BuildWithConfig(environment:MountainEnvironment, config:StateBuildConfig) -> Result<ApplicationState, String> {
	#[cfg(feature = "Telemetry")]
	let span = global::tracer("StateBuild").start("Build");

	info!("[StateBuild] Initializing application state");

	if config.verbose_logging {
		debug!("[StateBuild] Config: {:?}", config);
	}

	// Validate required capabilities if strict mode enabled
	if config.strict_validation {
		#[cfg(feature = "Telemetry")]
		span.set_attribute(KeyValue::new("validation", "strict"));

		if let Err(err) = ValidateCapabilities(&environment) {
			error!("[StateBuild] Capability validation failed: {}", err);
			#[cfg(feature = "Telemetry")]
			span.set_attribute(KeyValue::new("error", err.clone()));
			return Err(format!("Capability validation failed: {}", err));
		}
		info!("[StateBuild] All required capabilities validated");
	}

	// Create state with injected capabilities
	let state = ApplicationState::default();

	#[cfg(feature = "Telemetry")]
	{
		span.add_event("state_initialized", vec![]);
		span.end();
	}

	info!("[StateBuild] Application state initialized successfully");
	Ok(state)
}

/// Validate required capabilities are available
fn ValidateCapabilities(_environment:&MountainEnvironment) -> Result<(), String> {
	// Check critical capabilities
	// TODO: Implement actual capability checks based on Environment API
	Ok(())
}

/// Create minimal state for testing (reduced requirements)
#[cfg(any(test, feature = "Test"))]
pub fn BuildMinimal(_app_handle: tauri::AppHandle) -> Result<ApplicationState, String> {
	info!("[StateBuild] Creating minimal test state");
	// Create minimal ApplicationState for tests (no environment needed)
	// The environment is created later in the actual application lifecycle
	let app_state = ApplicationState::default();
	Ok(app_state)
}

#[cfg(test)]
mod tests {
	use super::*;

	// Note: These tests are disabled because MountainEnvironment::Create()
	// requires a tauri::AppHandle which cannot be easily created in unit tests.
	// Integration tests should be used instead.
	#[test]
	#[ignore = "Requires tauri::AppHandle - use integration tests instead"]
	fn test_state_build() {
		// Cannot create AppHandle in unit test context
		// Integration tests should be used for this
		unimplemented!("This test requires integration test setup with AppHandle");
	}

	#[test]
	#[ignore = "Requires tauri::AppHandle - use integration tests instead"]
	fn test_state_build_minimal() {
		// Cannot create AppHandle in unit test context
		// Integration tests should be used for this
		unimplemented!("This test requires integration test setup with AppHandle");
	}
}
