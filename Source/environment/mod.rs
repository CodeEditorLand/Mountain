//! Provides the concrete implementation of the application's environment.
//!
//! This module contains the `MountainEnvironment` struct and all of its
//! implementations of the provider traits defined in the `Common` crate,
//! organized into domain-specific modules.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// --- Main Environment Struct ---
pub mod MountainEnvironment;

// --- Provider Trait Implementations (organized by domain) ---
pub mod CommandProvider;
pub mod ConfigurationProvider;
pub mod CustomEditorProvider;
pub mod DiagnosticProvider;
pub mod DocumentProvider;
pub mod FsProvider;
pub mod IpcProvider;
pub mod LanguageFeatureProvider;
pub mod OutputProvider;
pub mod ScmProvider;
pub mod SecretProvider;
pub mod StatusBarProvider;
pub mod StorageProvider;
pub mod SyncProvider;
pub mod TerminalProvider;
pub mod TestProvider;
pub mod TreeViewProvider;
pub mod UiProvider;
pub mod WebviewProvider;
pub mod WorkspaceProvider;

// --- Internal Utilities ---
mod Utils;
