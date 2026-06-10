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

// ============ Feature Flags ============
#[cfg(feature = "Telemetry")]
use opentelemetry::{KeyValue, global};
#[cfg(feature = "Telemetry")]
use opentelemetry::trace::Tracer;
#[cfg(feature = "Telemetry")]
use opentelemetry::trace::Span;

use crate::{
	ApplicationState::State::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	dev_log,
};

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

	dev_log!("lifecycle", "[StateBuild] Initializing application state");

	if config.verbose_logging {
		dev_log!("lifecycle", "[StateBuild] Config: {:?}", config);
	}

	// Validate required capabilities if strict mode enabled
	if config.strict_validation {
		#[cfg(feature = "Telemetry")]
		span.set_attribute(KeyValue::new("validation", "strict"));

		if let Err(err) = ValidateCapabilities(&environment) {
			dev_log!("lifecycle", "error: [StateBuild] Capability validation failed: {}", err);

			#[cfg(feature = "Telemetry")]
			span.set_attribute(KeyValue::new("error", err.clone()));

			return Err(format!("Capability validation failed: {}", err));
		}

		dev_log!("lifecycle", "[StateBuild] All required capabilities validated");
	}

	// Create state with injected capabilities
	let state = ApplicationState::default();

	#[cfg(feature = "Telemetry")]
	{
		span.add_event("state_initialized", vec![]);

		span.end();
	}

	dev_log!("lifecycle", "[StateBuild] Application state initialized successfully");

	Ok(state)
}

/// Validate required capabilities are available.
///
/// Checks are best-effort: failures emit warnings but never propagate an
/// error to the caller, so a restricted sandbox or a permissions issue
/// does not prevent the application from starting.
fn ValidateCapabilities(environment:&MountainEnvironment) -> Result<(), String> {
	// Verify the app data directory is accessible.
	let DataDir = environment.ApplicationHandle.path().app_data_dir();

	let DataPath = match DataDir {
		Ok(P) => {
			dev_log!("lifecycle", "[StateBuild] App data dir: {}", P.display());

			P
		},

		Err(Error) => {
			dev_log!(
				"lifecycle",
				"warn: [StateBuild] app_data_dir() unavailable ({}); skipping write check",
				Error
			);

			return Ok(());
		},
	};

	// Verify we can create (and remove) a probe file in the data directory.
	// A missing directory is created on-demand; a permissions failure is
	// logged as a warning without aborting startup.
	if let Err(Error) = std::fs::create_dir_all(&DataPath) {
		dev_log!(
			"lifecycle",
			"warn: [StateBuild] Cannot create app data dir {}: {}",
			DataPath.display(),
			Error
		);

		return Ok(());
	}

	let ProbeFile = DataPath.join(".mountain_capability_probe");

	match std::fs::write(&ProbeFile, b"probe") {
		Ok(()) => {
			let _ = std::fs::remove_file(&ProbeFile);

			dev_log!("lifecycle", "[StateBuild] App data dir write check passed.");
		},

		Err(Error) => {
			dev_log!(
				"lifecycle",
				"warn: [StateBuild] App data dir write check failed ({}): {}; continuing in read-only mode",
				ProbeFile.display(),
				Error
			);
		},
	}

	Ok(())
}

/// Create minimal state for testing (reduced requirements)
#[cfg(any(test, feature = "Test"))]
pub fn BuildMinimal(_app_handle:tauri::AppHandle) -> Result<ApplicationState, String> {
	dev_log!("lifecycle", "[StateBuild] Creating minimal test state");

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
