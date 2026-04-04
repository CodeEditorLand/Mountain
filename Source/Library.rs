#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

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
//! - [`ProcessManagement`]: Sidecar process lifecycle (launch, monitor, restart)
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
/// Centralized error handling system
pub mod Error;

pub mod ApplicationState;

pub mod Environment;

pub mod RunTime;

// Communication
pub mod IPC;

pub mod Air;

pub mod Vine;

pub mod RPC;

// Services
pub mod ProcessManagement;

pub mod FileSystem;

pub mod ExtensionManagement;

// Commands
pub mod Command;

pub mod Track;

pub mod Workspace;

// Entry Point
pub mod Binary;

