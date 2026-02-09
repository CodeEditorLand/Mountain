//! # Main Module (Binary)
//!
//! ## RESPONSIBILITIES
//!
//! Main application orchestration providing entry point, command handlers,
//! lifecycle management, and tray integration for the Mountain desktop
//! application.
//!
//! This module serves as the primary entry point for the application and
//! coordinates all initialization, execution, and shutdown operations.
//!
//! ## ARCHITECTURAL ROLE
//!
//! The Main module is the **orchestration layer** in Mountain's architecture:
//!
//! ```text
//! main.rs ──► Binary::Main (Entry::Fn)
//!                      │
//!                      ├─► Entry::Fn() (Main entry point)
//!                      ├─► IPCCommands (Tauri command handlers)
//!                      ├─► AppLifecycle (Setup and initialization)
//!                      └─► Tray (System tray icon management)
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **Entry** (`Entry::Fn`): Main application entry point exported as
//!   `Binary::Main::Fn()`
//! - **IPCCommands**: All Tauri command handlers for frontend-backend
//!   communication
//!   - Workbench configuration commands
//!   - IPC messaging commands
//!   - Wind desktop configuration commands
//!   - Configuration management commands
//!   - Status reporting commands
//!   - Performance monitoring commands
//!   - Collaboration commands
//!   - Document sync commands
//! - **AppLifecycle**: Application lifecycle management and setup
//! - **Tray**: System tray icon switching based on theme
//!
//! ## EXPORTS
//!
//! - `pub use Entry::Fn as Main`: Main entry point
//! - All IPC command functions are exported from IPCCommands
//! - `AppLifecycleSetup`: Application setup function from AppLifecycle
//! - `SwitchTrayIcon`: Tray icon switching command from Tray
//!
//! ## ERROR HANDLING
//!
//! - Entry point panics on fatal errors (Tokio runtime, Tauri build)
//! - IPC commands return `Result<serde_json::Value, String>`
//! - Lifecycle setup returns `Result<(), Box<dyn std::error::Error>>`
//! - Non-critical failures are logged but don't prevent operation
//!
//! ## LOGGING
//!
//! Comprehensive logging throughout with checkpoints at key stages.
//! Logs use standardized prefixes: `[Boot]`, `[Lifecycle]`, `[IPC]`, `[UI]`.
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Async initialization spawned after main setup
//! - Compile-time resource embedding for tray icons
//! - Minimal allocation overhead with efficient error handling
//!
//! ## TODO
//! - [ ] Add comprehensive error recovery mechanism
//! - [ ] Implement startup progress indicator
//! - [ ] Add graceful degradation for service failures
//! - [ ] Implement performance metrics collection

/// Main application entry point and orchestration.
///
/// Contains the `Fn()` function which is the primary entry point for the
/// Mountain
// desktop application. This function creates the Tokio runtime, initializes
// application state, sets up the Tauri builder, and runs the application.
pub mod Entry;

/// IPC command handlers.
///
/// Contains all Tauri command handlers that provide the frontend-backend
/// communication bridge. Commands include workbench configuration, IPC
/// messaging,
// Wind desktop integration, configuration management, status reporting,
// performance monitoring, collaboration, and document synchronization.
pub mod IPCCommands;

/// Application lifecycle management.
///
/// Contains the `AppLifecycleSetup()` function which handles all initialization
// during the Tauri setup hook, including tray initialization, command
// registration, IPC server setup, window creation, environment configuration,
// and async service initialization.
pub mod AppLifecycle;

/// System tray commands.
///
/// Contains the `SwitchTrayIcon()` Tauri command for switching the system tray
// icon based on the current theme (light/dark mode).
pub mod Tray;

// --- Re-exports ---

/// Main application entry point.
///
/// Exported as `Binary::Main::Fn()` and can be called from main.rs
/// to start the Mountain desktop application.
pub use Entry::Fn as Main;
