//! # Status Reporter Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides monitoring and health checking for the IPC layer.
//! It reports Mountain's IPC status to Sky and enables real-time observability.
//!
//! ## ARCHITECTURAL ROLE
//! This module is the observability layer in the IPC architecture, providing
//! health monitoring and metrics reporting.
//!
//! ## KEY COMPONENTS
//!
//! - **Reporter**: Main StatusReporter orchestrator
//!
//! ## ERROR HANDLING
//! All operations return Result types with descriptive error messages.
//!
//! ## LOGGING
//! Info-level for status events, debug for health checks, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient metrics collection
//! - Batched reporting to reduce overhead
//! - Sampling for high-frequency metrics
//!
//! ## TODO
//! - Add custom health check endpoints
//! - Implement metrics dashboard
//! - Support alert thresholds
//! - Add historical data tracking

// Re-export the original file for backward compatibility
pub use crate::Element::Mountain::Source::IPC::StatusReporter as Reporter;
