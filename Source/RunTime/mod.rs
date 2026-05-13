//! Effect execution engine for the Mountain application. Provides the runtime
//! environment for executing effects through the Echo scheduler.

// --- Sub-modules ---

/// Application runtime module containing the struct definition.
/// The struct is accessible as
/// `RunTime::ApplicationRunTime::ApplicationRunTime`.
pub mod ApplicationRunTime;

/// Effect execution logic.
pub mod Execute;

/// Service shutdown and lifecycle management.
pub mod Shutdown;
