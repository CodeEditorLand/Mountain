
//! Lifecycle state of a test run: Queued → Running → terminal
//! (Passed / Failed / Skipped / Errored). Used both as the run-level
//! aggregate and the per-test-result status.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enum {
	Queued,

	Running,

	Passed,

	Failed,

	Skipped,

	Errored,
}
