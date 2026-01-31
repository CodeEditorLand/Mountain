// ============================================================================
// File: Mountain/Source/Command/mod.rs
// ============================================================================
// This module follows the Land ecosystem's PascalCase naming convention.
// See: https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//
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

#![allow(non_snake_case, non_camel_case_types)]

pub mod Bootstrap;
pub mod Keybinding;
pub mod LanguageFeature;
pub mod SourceControlManagement;
pub mod TreeView;
