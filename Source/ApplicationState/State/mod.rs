//! # State — Domain-specific state groups
//!
//! Each sub-module holds a slice of the overall application state, organized
//! by concern:
//!
//! - **WorkspaceState**: folders, trust, active document
//! - **ConfigurationState**: settings and memento storage
//! - **ExtensionState**: registry, providers, scanned extensions
//! - **FeatureState**: diagnostics, documents, terminals, webviews, etc.
//! - **UIState**: pending UI requests and state
//!
//! ## Sub-modules
//!
//! - [`WorkspaceState`]: Workspace folder and trust management
//! - [`ConfigurationState`]: Configuration and storage state
//! - [`ExtensionState`]: Extension registry, provider registration, scanned
//!   extensions
//! - [`FeatureState`]: Feature-specific state (documents, terminals, webviews,
//!   etc.)
//! - [`UIState`]: User interface request state
//! - [`ApplicationState`]: Main state container aggregating all sub-states

/// Workspace folder and trust management.
pub mod WorkspaceState;

/// Configuration and storage state.
pub mod ConfigurationState;

/// Extension management state: registry, provider registration, scanned
/// extensions.
pub mod ExtensionState;

/// Feature-specific state: diagnostics, documents, terminals, webviews,
/// markers, etc.
pub mod FeatureState;

/// User interface request state.
pub mod UIState;

/// Main ApplicationState container aggregating all sub-states.
pub mod ApplicationState;
