#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	unused_imports,
	unused_variables,
	unused_assignments
)]

//! # Mountain: Native Backend for Code Editor Land
//!
//! Mountain replaces Electron with Rust and Tauri. It manages windows, file
//! systems, processes, and extensions at native speed. Where Electron takes
//! milliseconds, Mountain responds in microseconds.
//!
//! ## What Mountain Does
//!
//! - **Hosts the editor UI** via Tauri webview (no Chromium process overhead)
//! - **Runs VS Code extensions** by managing the Cocoon sidecar over gRPC
//! - **Handles file I/O** through native async Rust (tokio), not Node.js `fs`
//! - **Manages terminals** via native PTY (`portable-pty`), not shell wrappers
//! - **Stores secrets** in the OS keychain (`keyring` crate), not plaintext
//!
//! ## Architecture
//!
//! Mountain uses a declarative effect system defined in `Common`. Business
//! logic is expressed as `ActionEffect`s executed by the `ApplicationRunTime`.
//! All state lives in a single thread-safe `ApplicationState` managed by Tauri.
//!
//! ```text
//! Wind/Sky (UI) ──Tauri commands──> Mountain ──gRPC──> Cocoon (extensions)
//!                                      │
//!                                      ├── Environment providers (file, process, terminal)
//!                                      ├── ApplicationRunTime (effect executor)
//!                                      └── ApplicationState (shared state)
//! ```
//!
//! ## Module Layout
//!
//! ### Core Infrastructure
//! - [`ApplicationState`]: Centralized, thread-safe state for the entire app
//! - [`Environment`]: Capability providers (file system, processes, extensions)
//! - [`RunTime`]: Effect execution engine that runs `ActionEffect` pipelines
//!
//! ### Communication
//! - [`IPC`]: Inter-process communication primitives
//! - [`Air`]: Client for the background daemon (updates, crypto signing)
//! - [`Vine`]: gRPC server/client for Cocoon extension host communication
//! - [`RPC`]: Remote procedure call service implementations
//!
//! ### Services
//! - [`ProcessManagement`]: Sidecar process lifecycle (launch, monitor,
//!   restart)
//! - [`FileSystem`]: Native TreeView provider for the File Explorer
//! - [`ExtensionManagement`]: Extension discovery, scanning, and activation
//!
//! ### Commands
//! - [`Command`]: Native command handlers (file, edit, view, terminal)
//! - [`Track`]: Central command dispatcher routing UI requests to providers
//! - [`Workspace`]: `.code-workspace` file parsing and multi-root support
//!
//! ## Related Crates
//!
//! | Crate | Role |
//! |---|---|
//! | `Common` | Abstract traits and DTOs that Mountain implements |
//! | `Echo` | Work-stealing task scheduler used by Mountain's runtime |
//! | `Air` | Background daemon that Mountain communicates with |
//!
//! ## Getting Started
//!
//! Mountain builds as part of the Land monorepo:
//! ```bash
//! cargo build -p Mountain
//! ```
//!
//! Full setup: <https://github.com/CodeEditorLand/Land>

// Core Infrastructure

/// Centralized, thread-safe application state managed by Tauri.
pub mod ApplicationState;

/// Capability providers: file system, process, terminal, and extension host.
pub mod Environment;

/// Effect execution engine that drives `ActionEffect` pipelines.
pub mod RunTime;

// Communication

/// Inter-process communication primitives.
pub mod IPC;

/// gRPC server and client for Cocoon extension host communication.
pub mod Vine;

/// Remote procedure call service implementations.
pub mod RPC;

// Services

/// Sidecar process lifecycle: launch, monitor, and restart.
pub mod ProcessManagement;

/// Native TreeView provider for the File Explorer.
pub mod FileSystem;

/// Extension discovery, scanning, and activation.
pub mod ExtensionManagement;

// Commands

/// Native command handlers for file, edit, view, and terminal operations.
pub mod Command;

/// Central command dispatcher routing UI requests to the correct provider.
pub mod Track;

/// `.code-workspace` file parsing and multi-root workspace support.
pub mod Workspace;

/// Emits a single ISO-timestamped boot banner listing all compiled-in tier
/// values.
pub mod LandFixTier;

/// Binary entry points for desktop and mobile builds.
pub mod Binary;

/// Engine-level interception layer (gated behind TierShim env var).
pub mod Shim;

/// Main entry point for both mobile and desktop builds.
#[allow(unexpected_cfgs)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() { Binary::Main::Entry::Fn(); }
