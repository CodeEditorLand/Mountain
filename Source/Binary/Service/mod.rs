#![allow(non_snake_case)]

//! # Binary::Service
//!
//! External service startup functions called from `Binary::Main::AppLifecycle`.
//! Each sub-module owns the full launch sequence for one service
//! and exposes a single async `Fn()` entry point.

/// Start the Vine notification broadcast service.
pub mod VineStart;

/// Start the Cocoon sandboxed-extension host service.
pub mod CocoonStart;

/// Load and validate the initial application configuration from disk.
pub mod ConfigurationInitialize;

/// Start the Air real-time collaboration service.
pub mod AirStart;
