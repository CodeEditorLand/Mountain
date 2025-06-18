// @module Environment
// @description Provides the concrete implementation of the application's
// Environment.
//
// This module contains the `MountainEnvironment` struct and all of its
// implementations of the provider traits defined in the `Common` crate,
// organized into domain-specific modules. This new structure uses composition,
// where the main `MountainEnvironment` holds instances of sub-Environments,
// promoting better separation of concerns.

#![allow(non_snake_case, non_camel_case_types)]

// --- Main Environment Struct ---
pub mod MountainEnvironment;

// --- Provider Trait Implementations (organized by domain) ---
// Each of these modules will define a small Environment struct that holds an
// `ApplicationHandle` and implements the relevant provider traits from
// `Common`.
pub mod CommandProvider;
pub mod ConfigurationProvider;
pub mod CustomEditorProvider;
pub mod DiagnosticProvider;
pub mod DocumentProvider;
pub mod FileSystemProvider;
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
pub mod WebViewProvider;
pub mod WorkspaceProvider;

// --- Internal Utilities ---
// Shared helpers for provider implementations.
pub mod Utility;
