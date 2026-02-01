//! # Initialize
//!
//! Initialization utilities for the Mountain binary.
//!
//! ## RESPONSIBILITIES
//!
//! ### Module Organization
//! - Export all initialization modules
//! - Provide init utilities for startup
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Top-level init module in Binary subsystem
//! - Provides startup initialization utilities
//!
//! ### Dependencies
//! - All init submodules
//!
//! ### Dependents
//! - Binary: Uses init utilities during startup

pub mod RuntimeBuild;
pub mod CliParse;
pub mod StateBuild;
pub mod PortSelector;
pub mod LogLevel;
