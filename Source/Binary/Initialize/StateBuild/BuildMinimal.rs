//! `StateBuild::BuildMinimal`

use std::sync::Arc;
use opentelemetry::{KeyValue, global};
use opentelemetry::trace::Tracer;
use opentelemetry::trace::Span;
use crate::{
	ApplicationState::Struct::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	dev_log,
};

/// Create minimal state for testing (reduced requirements)
#[cfg(any(test, feature = "Test"))]
pub fn Fn(_app_handle:tauri::AppHandle) -> Result<ApplicationState, String> {
	dev_log!("lifecycle", "[StateBuild] Creating minimal test state");

	// Create minimal ApplicationState for tests (no environment needed)
	// The environment is created later in the actual application lifecycle
	let app_state = ApplicationState::default();

	Ok(app_state)
}
