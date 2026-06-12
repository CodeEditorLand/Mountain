//! # State
//!
//! Domain-specific state groups for the Mountain application.
//! Each sub-module holds a slice of the overall application state,
//! organized by concern:
//!
//! - WorkspaceState: folders, trust, active document
//! - ConfigurationState: settings and memento storage
//! - ExtensionState: registry, providers, scanned extensions
//! - FeatureState: diagnostics, documents, terminals, webviews, etc.
//! - UIState: pending UI requests and state

//! Workspace state management.
/// Workspacestate module.
pub mod WorkspaceState;

/// Configuration and storage state.
pub mod ConfigurationState;

/// Extension management state.
pub mod ExtensionState;

/// Feature-specific state management.
pub mod FeatureState;

/// User interface request state.
pub mod UIState;

/// Main ApplicationState container for backward compatibility.
pub mod ApplicationState;
