//! # Shutdown (RunTime)
//!
//! ## RESPONSIBILITIES
//!
//! Service shutdown and lifecycle management for graceful application termination.
//! Coordinates cleanup of all application services with error recovery.
//!
//! Provides comprehensive shutdown operations through impl blocks on ApplicationRunTime:
//! - Main shutdown orchestration
//! - Cocoon sidecar shutdown with retry
//! - Terminal disposal
//! - Application state persistence
//! - Pending operation cleanup
//!
//! ## ARCHITECTURAL ROLE
//!
//! The lifecycle manager in Mountain's architecture that ensures clean
//! application termination and state persistence.
//!
//! ## KEY COMPONENTS
//!
//! - **Shutdown**: Core shutdown functions implemented on ApplicationRunTime
//!
//! ## ERROR HANDLING
//!
//! All shutdown operations use error recovery to continue cleanup even when
//! individual services fail. Errors are collected and reported without crashing.
//! Multi-attempt retry for critical operations like Cocoon shutdown.
//!
//! ## LOGGING
//!
//! Uses log crate with appropriate severity levels:
//! - `info`: Shutdown initiation and completion
//! - `debug`: Detailed operation steps
//! - `warn`: Recoverable errors during shutdown
//! - `error`: Failed operations (but continues shutdown)
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Shutdown operations are optimized to complete quickly
//! - Parallel cleanup where safe (terminal disposal)
//! - Minimal blocking during state persistence
//! - Uses timeout recovery to prevent hanging
//!
//! ## TODO
//!
//! Medium Priority:
//! - [ ] Add shutdown progress reporting
//! - [ ] Implement shutdown timeout per service
//! - [ ] Add shutdown cancellation support
//!
//! Low Priority:
//! - [ ] Add shutdown metrics collection
//! - [ ] Implement service dependency ordering
//! - [ ] Add shutdown rollback for partial failures

// --- Sub-modules ---

/// Shutdown orchestration and service cleanup implemented on ApplicationRunTime.
pub mod Shutdown;

// Note: The shutdown functions (Shutdown, ShutdownWithRecovery, ShutdownCocoonWithRetry,
// DisposeTerminalsSafely, SaveApplicationState, FlushPendingOperations) are now implemented
// as methods on the ApplicationRunTime struct in Shutdown.rs
