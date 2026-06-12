//! Lifecycle state of a test run: Queued → Running → terminal
//! (Passed / Failed / Skipped / Errored). Used both as the run-level
//! aggregate and the per-test-result status.

use serde::{Deserialize, Serialize};

/// Lifecycle state of a test run or individual test result.
///
/// Transitions: Queued → Running → one of the terminal states
/// (Passed, Failed, Skipped, Errored).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enum {
	/// Test is queued and awaiting execution.
	Queued,

	/// Test is currently executing.
	Running,

	/// Test passed without errors.
	Passed,

	/// Test failed (assertion failure).
	Failed,

	/// Test was skipped (e.g. due to a filter).
	Skipped,

	/// Test encountered an error during setup or execution.
	Errored,
}
