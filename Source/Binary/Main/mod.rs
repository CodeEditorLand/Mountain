#![allow(non_snake_case)]

//! # Binary::Main
//!
//! Application orchestration layer providing entry point, IPC command
//! handlers, lifecycle management, and tray integration for the Mountain
//! desktop application.
//!
//! ## Module Layout
//!
//! ```text
//! main.rs --> Binary::Main::Entry::Fn()
//!                      |
//!                      +-> Entry       (Tokio runtime creation, Tauri builder)
//!                      +-> IPCCommands (all #[tauri::command] handlers)
//!                      +-> AppLifecycle(setup hook: tray, IPC server, window)
//!                      +-> Tray        (SwitchTrayIcon Tauri command)
//! ```
//!
//! ## Error Handling
//!
//! - `Entry::Fn` panics on fatal errors (Tokio runtime failure, Tauri build).
//! - IPC commands return `Result<serde_json::Value, String>`.
//! - Lifecycle setup returns `Result<(), Box<dyn std::error::Error>>`.
//! - Non-critical failures are logged but do not prevent operation.
//!
//! ## Logging
//!
//! Log prefixes used throughout: `[Boot]`, `[Lifecycle]`, `[IPC]`, `[UI]`.
//!
//! No `pub use` re-exports - callers use the full path
//! `Binary::Main::Entry::Fn()` directly.
//!
//! ## Planned Work
//!
//! - Comprehensive error recovery mechanism
//! - Startup progress indicator
//! - Graceful degradation for service failures
//! - Performance metrics collection

/// Main application entry point.
///
/// Exports `Fn()` which creates the Tokio runtime, initializes application
/// state, constructs the Tauri builder, and runs the event loop.
pub mod Entry;

/// IPC command handlers.
///
/// All `#[tauri::command]` functions providing the frontend-to-backend
/// invoke bridge: workbench configuration, IPC messaging, Wind desktop
/// integration, configuration management, status reporting, performance
/// monitoring, collaboration, and document synchronization.
pub mod IPCCommands;

/// Application lifecycle management.
///
/// Exports `AppLifecycleSetup()` which runs inside the Tauri setup hook:
/// tray initialization, command registration, IPC server setup, window
/// creation, environment configuration, and async service startup.
pub mod AppLifecycle;

/// System tray commands.
///
/// Exports the `SwitchTrayIcon()` Tauri command for switching the system
/// tray icon based on the active theme (light or dark mode).
pub mod Tray;
