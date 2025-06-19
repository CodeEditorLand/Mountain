//! # Environment Module
//!
//! Provides the concrete implementation of the application's Environment.
//!
//! This module contains the `MountainEnvironment` struct and all of its
//! implementations of the provider traits defined in the `Common` crate. Each
//! provider implementation is organized into its own file (e.g.,
//! `CommandProvider.rs`, `FileSystemProvider.rs`) for clarity and separation
//! of concerns.

#![allow(non_snake_case, non_camel_case_types)]

// --- Main Environment Struct ---
pub mod MountainEnvironment;

// --- Provider Trait Implementations (organized by domain) ---
pub mod CommandProvider;
pub mod ConfigurationProvider;
pub mod CustomEditorProvider;
pub mod DiagnosticProvider;
pub mod DocumentProvider;
pub mod FileSystemProvider;
pub mod IPCProvider;
pub mod LanguageFeatureProvider;
pub mod OutputProvider;
pub mod SecretProvider;
pub mod SourceControlManagementProvider;
pub mod StatusBarProvider;
pub mod StorageProvider;
pub mod SynchronizationProvider;
pub mod TerminalProvider;
pub mod TestProvider;
pub mod TreeViewProvider;
pub mod UserInterfaceProvider;
pub mod WebViewProvider;
pub mod WorkspaceProvider;

// --- Internal Utilities ---
// Shared helpers for provider implementations.
pub mod Utility;
