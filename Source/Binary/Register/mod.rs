//! # Binary::Register
//!
//! Startup registration steps invoked from `Binary::Main::AppLifecycle`.
//! Each sub-module owns one registration concern - command wiring,
//! IPC server setup, status reporting, Wind sync, or advanced features -
//! and exposes a single `Fn()` entry point.

/// Register all `#[tauri::command]` handlers with the Tauri invoke handler.
pub mod CommandRegister;

/// Bind and start the internal IPC server socket.
pub mod IPCServerRegister;

/// Attach the background status-reporter task to the Tokio runtime.
pub mod StatusReporterRegister;

/// Register advanced feature flags and capability handlers.
pub mod AdvancedFeaturesRegister;

/// Register the Wind desktop configuration sync task.
pub mod WindSyncRegister;
