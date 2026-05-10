#![allow(non_snake_case)]

//! # Binary::Initialize
//!
//! Pre-Tauri startup utilities invoked before the event loop begins.
//! Covers Tokio runtime construction, CLI argument parsing, application
//! state assembly, port selection, and log-level configuration.

/// Build and return the configured multi-thread Tokio runtime.
pub mod RuntimeBuild;

/// Parse and validate command-line arguments into a typed configuration struct.
pub mod CliParse;

/// Assemble the initial `MountainState` from disk and defaults.
pub mod StateBuild;

/// Select an available TCP port for the IPC server at startup.
pub mod PortSelector;

/// Resolve the active log level from CLI flags and environment variables.
pub mod LogLevel;
