#![allow(non_snake_case)]

//! # Binary::Debug
//!
//! Debug tracing infrastructure for the Mountain binary.
//! Exports the `TraceLog` module which provides the `TraceStep!` macro
//! for annotated execution-path logging; all output is gated behind the
//! active log level and compiles to a no-op in release builds.

/// Execution-path trace logging macro and supporting utilities.
pub mod TraceLog;
pub mod WebkitServer;
