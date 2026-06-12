//! Bridge between the declarative `ActionEffect` system and the Echo
//! work-stealing scheduler. Three entry points: bare `Run` (trait method),
//! `RunWithTimeout`, and `RunWithRetry`.

/// Direct effect execution (trait method).
pub mod Run;

/// Retry-capable effect execution.
pub mod RunWithRetry;

/// Timeout-bounded effect execution.
pub mod RunWithTimeout;
