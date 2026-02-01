// ============================================================================
// File: Mountain/Source/Command/mod.rs
// ============================================================================
// # Command Module
//
// This module is responsible for defining and registering all native
// (Rust-implemented) commands for the Mountain application. Commands are
// organized by functionality and registered at startup.
//
// ## Module Structure:
// - **Bootstrap**: Registers all native commands and providers at startup
// - **LanguageFeature**: Handles LSP-related commands (hover, completion, etc.)
// - **TreeView**: Manages tree view UI commands
// - **Keybinding**: Handles keybinding-related commands
// - **SourceControlManagement**: SCM (git) command implementations
//
// ## VSCode Reference:
// - vs/workbench/services/commands/common/commandService.ts
// - vs/platform/commands/common/commands.ts
//
// ============================================================================

pub mod Bootstrap;
pub mod Keybinding;
pub mod LanguageFeature;
pub mod SourceControlManagement;
pub mod TreeView;
