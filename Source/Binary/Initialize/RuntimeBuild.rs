//! # RuntimeBuild
//!
//! Builds the Tokio runtime for async execution.
//!
//! ## RESPONSIBILITIES
//!
//! ### Runtime Construction
//! - Create multi-threaded Tokio runtime
//! - Enable all runtime features (IO, time, etc.)
//! - Validate runtime construction
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Early initialization component in Binary subsystem
//! - Provides async execution environment
//!
//! ### Dependencies
//! - tokio: Async runtime
//!
//! ### Dependents
//! - Fn() main entry point: Uses runtime for async execution
//!
//! ## SECURITY
//!
//! ### Considerations
//! - No security impact (runtime construction only)
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Runtime construction is one-time cost at startup
//! - Multi-threaded config maximizes CPU utilization

use tokio::runtime::Builder;

/// Build the Tokio runtime for async execution.
///
/// Creates a multi-threaded Tokio runtime with all features enabled.
/// This is required for all async operations in the application.
///
/// # Returns
///
/// Returns a configured Tokio runtime.
///
/// # Panics
///
/// Panics if runtime construction fails.
pub fn Build() -> tokio::runtime::Runtime {
	Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("FATAL: Cannot build Tokio runtime.")
}
