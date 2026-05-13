#![allow(non_snake_case)]
#![allow(unused_imports, unused_variables)]

//! # Binary
//!
//! Main entry point and initialization for the Mountain desktop application.
//! Handles application startup, Tauri command registration, configuration,
//! and lifecycle management.
//!
//! ## Module Layout
//!
//! - [`Main`]: Application entry point and orchestration
//! - [`Build`]: Tauri application builder configuration
//! - [`Register`]: Command and service registration
//! - [`Service`]: Service initialization (Vine, Cocoon, Configuration)
//! - [`Initialize`]: Application state initialization
//! - [`IPC`]: IPC command handlers bridging the frontend invoke calls to Rust
//! - [`Tray`]: System tray integration
//! - [`Extension`]: Extension startup and management
//! - [`Shutdown`]: Graceful shutdown handling
//! - [`Debug`]: Debug and trace logging utilities
//!
//! ## Architectural Role
//!
//! ```text
//! main.rs --> Binary::Main (Entry) --> Build --> Register --> Initialize --> Services
//!                                    |           |            |             |
//!                                    v           v            v             v
//!                               AppLifecycle  Commands    Services    Vine/Cocoon
//!                                        |            |            |
//!                                  IPCCommands  IPCBridge  ProcessMgmt
//! ```
//!
//! ## Design Principles
//!
//! 1. **Single Entry Point**: One clear entry point for the application.
//! 2. **Lazy Initialization**: Services started only when needed.
//! 3. **Graceful Shutdown**: Clean shutdown of all services.
//! 4. **Error Resilience**: Graceful degradation on startup failures.
//!
//! No `pub use` re-exports - callers spell the full reverse-hierarchical
//! path (`Binary::Main::Entry::Fn`, `Binary::Build::LocalhostPlugin::Fn`,
//! etc.).
//!
//! ## Planned Work
//!
//! - Crash recovery mechanism
//! - Error dialog for startup failures
//! - Startup performance metrics
//! - Incremental service startup
//! - Service health checks during startup

/// Main application entry point and orchestration.
pub mod Main;

/// Tauri application builder configuration.
pub mod Build;

/// Command and service registration.
pub mod Register;

/// Service initialization (Vine, Cocoon, Configuration).
pub mod Service;

/// Application state initialization.
pub mod Initialize;

/// IPC command handlers bridging the frontend invoke calls to Rust.
pub mod IPC;

/// System tray integration.
pub mod Tray;

/// Extension startup and management.
pub mod Extension;

/// Graceful shutdown handling.
pub mod Shutdown;

/// Debug and trace logging utilities.
pub mod Debug;
