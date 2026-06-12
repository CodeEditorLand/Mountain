//! Per-test outcome: identifier, fully-qualified name, status, optional
//! duration in ms, and optional error/stack-trace pair populated on
//! failures.

use serde::{Deserialize, Serialize};

use crate::Environment::TestProvider::TestRunStatus;

/// Outcome of a single test execution.
///
/// Carries the test identifier, fully-qualified name, run status,
/// optional duration, and optional error details populated on failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	/// Unique identifier for this test within its controller.
	pub TestIdentifier:String,
	/// Fully-qualified human-readable test name.
	pub FullName:String,
	/// Execution status (Passed, Failed, Skipped, Errored).
	pub Status:TestRunStatus::Enum,
	/// Execution duration in milliseconds, if measured.
	pub DurationMs:Option<u64>,
	/// Error message string, populated on failure.
	pub ErrorMessage:Option<String>,
	/// Stack trace string, populated on failure.
	pub StackTrace:Option<String>,
}
