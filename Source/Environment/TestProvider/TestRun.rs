//! Active test run record: run identifier, owning controller, current
//! status, start instant, and per-test results keyed by test
//! identifier. Stored in `TestProviderState::Struct::ActiveRuns` for
//! the duration of the run.

use std::collections::HashMap;

use crate::Environment::TestProvider::{TestResult, TestRunStatus};

/// Record of an active (or recently completed) test run.
///
/// Tracks the run identifier, owning controller, current status, start
/// time, and per-test results. Stored in
/// `TestProviderState::Struct::ActiveRuns` for the run duration.
#[derive(Debug, Clone)]
pub struct Struct {
	/// Unique identifier for this test run.
	pub RunIdentifier:String,

	/// Identifier of the controller that owns this run.
	pub ControllerIdentifier:String,

	/// Current lifecycle status (Queued, Running, …).
	pub Status:TestRunStatus::Enum,

	/// Instant when this run started.
	pub StartedAt:std::time::Instant,

	/// Per-test outcomes keyed by test identifier.
	pub Results:HashMap<String, TestResult::Struct>,
}
