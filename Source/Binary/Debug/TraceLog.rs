#![allow(unused_imports)]

//! # TraceLog
//!
//! Provides debug tracing macro for fine-grained execution step tracking.
//!
//! ## RESPONSIBILITIES
//!
//! ### Macro Definition
//! - Define TraceStep macro for trace-level logging
//! - Provide low-intrusion debug checkpoint logging
//! - Support formatted trace messages for step-by-step execution
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Debug infrastructure module in Binary subsystem
//! - Cross-cutting concern available throughout the application
//!
//! ### Dependencies
//! - log: Logging framework
//! - trace: Trace level log filter
//!
//! ### Dependents
//! - All Binary subsystem modules for execution tracking
//! - Fn() main entry point for startup sequence logging
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Macro expands to trace-level logs, does not modify runtime behavior
//! - No security impact, purely diagnostic
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Logs at TRACE level, filtered out in production builds
//! - Zero runtime cost when RUST_LOG does not include TRACE level
//! - Minimal code generation overhead from macro expansion

use log::trace;

/// Logs a checkpoint message at TRACE level (for "every step" tracing).
///
/// This macro provides a low-intrusion way to trace execution flow through
/// the application startup and shutdown sequences. It expands to a single
/// trace!() call which incurs zero overhead when TRACE logging is disabled.
///
/// # Example
///
/// ```rust,ignore
/// TraceStep!("[Boot] [Runtime] Building Tokio runtime...");
/// TraceStep!("[Boot] [Setup] Configuration loaded: {}", ConfigPath);
/// ```
///
/// The macro accepts the same format arguments as the standard log!() macro:
/// - A literal format string
/// - Optional comma-separated values for format placeholders
#[macro_export]
macro_rules! TraceStep {
	($($arg:tt)*) => {{
		trace!($($arg)*);
	}};
}
