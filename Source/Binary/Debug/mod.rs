//! # Debug
//!
//! Debug tracing infrastructure for the Mountain binary.
//!
//! ## RESPONSIBILITIES
//!
//! ### Module Organization
//! - Export all debug-related modules
//! - Provide TraceStep macro for execution tracking
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Top-level debug module in Binary subsystem
//!
//! ### Dependencies
//! - log: Logging framework
//!
//! ### Dependents
//! - All Binary subsystem modules requiring debug tracing
//!
//! ## SECURITY
//!
//! ### Considerations
//! - All debug output is filtered by log level
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Zero-cost when debug logging disabled

pub mod TraceLog;
