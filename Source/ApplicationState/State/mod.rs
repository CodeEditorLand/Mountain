//! # State Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Contains all state management sub-modules for the Mountain application.
//! Each submodule represents a distinct domain-specific state area.
//!
//! ## KEY COMPONENTS
//! - WorkspaceState: Workspace folders, trust, active document
//! - ConfigurationState: Configuration, memento storage
//! - ExtensionState: Extension registry, providers, scanned extensions
//! - FeatureState: Diagnostics, documents, terminals, webviews, etc.
//! - UIState: Pending UI requests
//! - ApplicationState: Main state container (for backward compatibility)
//!
//! ## ARCHITECTURAL ROLE
//! The State module is the **state organization layer** that groups related
//! state components into logical domains:
//!
//! ```text
//! ApplicationState
//! │
//! ├── WorkspaceState      - Workspace folders, trust, active document
//! ├── ConfigurationState  - Configuration, memento storage
//! ├── ExtensionState      - Extension registry, providers, scanned extensions
//! ├── FeatureState        - Diagnostics, documents, terminals, webviews, etc.
//! └── UIState             - Pending UI requests
//! ```
//!
//! ## KEY COMPONENTS
//! - **WorkspaceState**: Workspace-related state
//! - **ConfigurationState**: Configuration and storage state
//! - **ExtensionState**: Extension management state (composite)
//! - **FeatureState**: Feature-specific state (composite)
//! - **UIState**: User interface request state
//!
//! ## ERROR HANDLING
//! All state operations use `Arc<Mutex<...>>` for thread-safety with proper
//! error handling via `MapLockError` helpers.
//!
//! ## LOGGING
//! State operations are logged at appropriate levels (debug, info, warn,
//! error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! -Avoid nested locks to prevent deadlocks
//! - Use `Arc` for shared ownership across threads
//!
//! ## TODO
//! - [ ] Add state validation invariants
//! - [ ] Implement state metrics collection
//! - [ ] Add state diffing for debugging

//! Workspace state management.
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

/// Re-export for backward compatibility
pub use ApplicationState::*;
