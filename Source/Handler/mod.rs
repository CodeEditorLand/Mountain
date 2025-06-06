// ---------------------------------------------------------------------------------------------
// Mountain Handlers Module 
// --------------------------------------------------------------------------------------------
// This file declares all public sub-modules within the `handlers` directory.
// Each sub-module typically contains logic for a specific domain or type of
// interaction, such as command execution, configuration management, document
// handling, IPC bridging, etc.
//
// These handlers are often called by:
// - `track.rs` (for RPC calls from sidecars or frontend commands).
// - `environment.rs` (provider trait implementations often delegate to these
//   handlers).
// - Tauri command handlers defined elsewhere (e.g., in `track.rs` or
//   `main.rs`).
// --------------------------------------------------------------------------------------------

// Declare all public sub-modules within the `handlers` directory.
// For these to compile, corresponding files (e.g., `commands.rs`, `config.rs`)
// or sub-directories with `mod.rs` (e.g., `commands/mod.rs`) must exist.

pub mod commands;
pub mod config;
pub mod diagnostics;
pub mod documents;
pub mod enablement;
pub mod error_utils;
pub mod extension_status;
pub mod language_features; // Added from snippets
pub mod native_fs;
pub mod output;
pub mod process_mgmt; // Added, was in the main lib.rs structure before
pub mod protocol;
pub mod proxy;
pub mod registry;
pub mod secrets;
pub mod sky_commands; // Added from snippets
pub mod sky_configuration; // Added from snippets
pub mod sky_dtos; // Added from snippets
pub mod sky_ipc_bridge; // Added from snippets
pub mod sky_ui_responses; // Was in the main lib.rs structure before
pub mod storage;
pub mod terminal;
pub mod ui;
pub mod workspace;
pub mod workspace_fs_api;

// Note: Modules like `enablement`, `native_fs`, `protocol`, `proxy`,
// `registry`, `ui`, and `workspace_fs_api` were in the original `lib.rs`'s
// handlers block. They are commented out here but should be uncommented if they
// are indeed part of the `handlers` directory structure.
