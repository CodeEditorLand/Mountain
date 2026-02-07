//! # Binary Module
//!
//! ## RESPONSIBILITIES
//!
//! Main entry point and initialization for the Mountain desktop application.
//! This module handles application startup, Tauri command registration,
//! configuration, and lifecycle management.
//!
//! ### Core Functions:
//! - **Application Entry**: Main application entry point
//! - **Tauri Setup**: Configure Tauri application builder
//! - **Command Registration**: Register all Tauri commands
//! - **IPC Bridge**: Set up IPC communication with frontend
//! - **Service Initialization**: Start Vine and Cocoon services
//! - **Tray Management**: Configure system tray
//! - **Lifecycle**: Handle application lifecycle events
//!
//! ## Architectural Role
//!
//! The Binary module is the **entry point** in Mountain's architecture:
//!
//! ```text
//! main.rs ──► Binary::Main (Entry) ──► Build ──► Register ──► Initialize ──► Services
//!                                    │            │             │             │
//!                                    ▼            ▼             ▼             ▼
//!                                AppLifecycle   Commands    Services    Vine/Cocoon
//!                                         │            │             │
//!                                   IPCCommands  IPCBridge   ProcessMgmt
//!```
//!
//! ### Design Principles:
//! 1. **Single Entry Point**: One clear entry point for the application
//! 2. **Lazy Initialization**: Services started only when needed
//! 3. **Graceful Shutdown**: Clean shutdown of all services
//! 4. **Error Resilience**: Graceful degradation on failures
//!
//! ## Key Components
//!
//! - **Main**: Application entry point and orchestration
//! - **Build**: Tauri builder configuration
//! - **Register**: Command and service registration
//! - **Service**: Service initialization (Vine, Cocoon)
//! - **Initialize**: Application state initialization
//! - **IPC**: IPC command handlers (14 commands)
//! - **Tray**: System tray integration
//! - **Extension**: Extension startup
//!
//! ## TODOs
//! High Priority:
//! - [x] Atomize Main.rs into submodules
//! - [ ] Add crash recovery mechanism
//! - [ ] Implement proper error dialog for startup failures
//!
//! Medium Priority:
//! - [ ] Add startup performance metrics
//! - [ ] Implement incremental service startup
//! - [ ] Add service health checks during startup
//!
//! Low Priority:
//! - [ ] Add startup progress indicator
//! - [ ] Implement startup animation
//! - [ ] Add startup sound

// --- Main Sub-module ---

/// Main application entry point and orchestration.
pub mod Main;

// --- Builder Sub-module ---

/// Tauri application builder configuration.
pub mod Build;

// --- Register Sub-module ---

/// Command and service registration.
pub mod Register;

// --- Service Sub-module ---

/// Service initialization (Vine, Cocoon, Configuration).
pub mod Service;

// --- Initialize Sub-module ---

/// Application state initialization.
pub mod Initialize;

// --- IPC Commands Sub-module ---

/// IPC command handlers (14 commands).
pub mod IPC;

// --- Tray Sub-module ---

/// System tray integration.
pub mod Tray;

// --- Extension Sub-module ---

/// Extension startup and management.
pub mod Extension;

// --- Shutdown Sub-module ---

/// Graceful shutdown handling.
pub mod Shutdown;

// --- Debug Sub-module ---

/// Debug and trace logging utilities.
pub mod Debug;

// --- Re-exports from Main sub-module for backward compatibility and convenience ---

use Main::{Entry, AppLifecycle, IPCCommands};

pub use Entry::Fn as Main;
pub use AppLifecycle::*;
// Note: IPCCommands is now a placeholder, commands are in Binary/IPC/*
// Note: Tray is now a placeholder, commands are in Binary/Tray/*

// --- Convenience re-exports from other sub-modules ---

pub use Build::{TauriBuild, WindowBuild, LoggingPlugin, LocalhostPlugin};
pub use Register::{CommandRegister, IPCServerRegister, StatusReporterRegister, AdvancedFeaturesRegister, WindSyncRegister};
pub use Service::{VineStart, CocoonStart, ConfigurationInitialize};
pub use Initialize::{CliParse, LogLevel, PortSelector, RuntimeBuild, StateBuild};
pub use Shutdown::{RuntimeShutdown, SchedulerShutdown};

// --- Tray re-exports from atomic modules ---

pub mod TrayModule {
	pub use super::Tray::EnableTray::enable_tray as EnableTray;
	pub use super::Tray::SwitchTrayIcon::SwitchTrayIcon;
}
