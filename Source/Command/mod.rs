//! # Command Module
//!
//! Defines and registers all native (Rust-implemented) commands for the

#![allow(unused_imports, unused_variables)]
//! Mountain application. Commands are organized by functionality and registered
//! at startup via the `Bootstrap` module.
//!
//! ## RESPONSIBILITIES
//!
//! - **Command Registration**: Register all Tauri command handlers with the
//!   invoke_handler
//! - **Module Organization**: Group commands by domain (LanguageFeature, SCM,
//!   etc.)
//! - **Handler Coordination**: Delegate command execution to appropriate
//!   providers
//! - **Error Propagation**: Convert provider errors to Tauri-compatible strings
//!
//! ## MODULE STRUCTURE
//!
//! - `Bootstrap`: Registers all native commands and providers at startup
//! - `LanguageFeature`: Handles LSP-related commands (hover, completion, etc.)
//! - `TreeView`: Manages tree view UI commands
//! - `Keybinding`: Handles keybinding-related commands
//! - `SourceControlManagement`: SCM (git) command implementations
//!
//! ## ARCHITECTURAL ROLE
//!
//! The Command module is the **UI-to-Backend bridge**:
//! ```text
//! Sky Frontend ──► Tauri invoke ──► Command Handler ──► Effect System ──► Providers
//! ```
//!
//! ### Design Patterns:
//! 1. **Command Pattern**: Each handler encapsulates a request
//! 2. **Facade Pattern**: Simple interface to complex backend operations
//! 3. **Error Translation**: Convert CommonError → String for Tauri
//!
//! ### VS Code Reference:
//! - `vs/workbench/services/commands/common/commandService.ts` - Command
//!   registry
//! - `vs/platform/commands/common/commands.ts` - Command definitions
//! - `vs/base/parts/ipc/common/ipc.ts` - IPC-based command execution
//!
//! ## COMMAND FLOW
//!
//! 1. **Registration**: All command handlers registered in Binary::Main via
//!    `tauri::generate_handler![]`
//! 2. **Invocation**: Frontend calls `app.invoke("command_name", args)`
//! 3. **Dispatch**: `DispatchLogic::DispatchFrontendCommand` routes to effect
//! 4. **Execution**: Effect runs via `ApplicationRunTime` with proper
//!    capabilities
//! 5. **Response**: Result serialized and returned to frontend
//!
//! ## ERROR HANDLING
//!
//! - All errors returned as `Result<T, String>` for Tauri compatibility
//! - Errors are logged with context before being returned
//! - Frontend receives descriptive error messages for user display
//!
//! ## PERFORMANCE
//!
//! - Commands are async to avoid blocking the UI thread
//! - Minimal serialization overhead (direct JSON)
//! - Effect system provides async execution with scheduler
//!
//! ## TODO
//!
//! - [ ] Add command metrics (invocation count, duration)
//! - [ ] Implement command throttling for high-frequency operations
//! - [ ] Add command permission validation
//! - [ ] Support command cancellation for long-running operations
//!
//! ## MODULE CONTENTS
//!
//! - [`Bootstrap`]: Command registration entry point
//! - [`Hover`]: Hover language feature (atomic structure example)
//! - [`LanguageFeature`]: LSP hover/completion/definition commands (legacy)
//! - [`TreeView`]: Tree view manipulation commands
//! - [`Keybinding`]: Keybinding resolution commands
//! - [`SourceControlManagement`]: Git/SCM commands

pub mod Bootstrap;

pub mod Hover; // Atomic structure (new)
pub mod Keybinding;

pub mod LanguageFeature;

pub mod SourceControlManagement;

pub mod TreeView;
