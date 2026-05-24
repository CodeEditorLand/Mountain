//! `StateBuild::Build`

use std::sync::Arc;
use opentelemetry::{KeyValue, global};
use opentelemetry::trace::Tracer;
use opentelemetry::trace::Span;
use crate::{
	ApplicationState::Struct::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	dev_log,
};

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
pub fn Fn(environment:MountainEnvironment) -> Result<ApplicationState, String> {
	BuildWithConfig(environment, StateBuildConfig::default())
}
