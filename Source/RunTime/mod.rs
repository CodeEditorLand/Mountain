//! RunTime — Effect execution engine for the Mountain application.
//!
//! Provides the runtime environment that drives `ActionEffect` pipelines
//! through the Echo scheduler.
//!
//! ## Sub-modules
//!
//! - [`ApplicationRunTime`]: Runtime struct definition and lifecycle
//! - [`Execute`]: Effect execution wrappers (retry, timeout, direct run)
//! - [`Shutdown`]: Graceful service shutdown and lifecycle management

/// Runtime struct definition (accessible as
/// `RunTime::ApplicationRunTime::ApplicationRunTime`).
pub mod ApplicationRunTime;

/// Effect execution wrappers: run, run with retry, run with timeout.
pub mod Execute;

/// Graceful service shutdown and lifecycle management (dispose terminals, flush
/// ops, save state).
pub mod Shutdown;
