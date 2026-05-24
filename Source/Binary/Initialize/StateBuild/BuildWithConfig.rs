//! `StateBuild::BuildWithConfig`

use std::sync::Arc;
use opentelemetry::{KeyValue, global};
use opentelemetry::trace::Tracer;
use opentelemetry::trace::Span;
use crate::{
	ApplicationState::Struct::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	dev_log,
};

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
pub fn Fn(environment:MountainEnvironment, config:StateBuildConfig) -> Result<ApplicationState, String> {
	#[cfg(feature = "Telemetry")]
	let span = global::tracer("StateBuild").Start("Build");

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
