//! # WorkspaceState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages workspace-related state including workspace folders, workspace
//! trust status, and the currently active document.
//!
//! ## ARCHITECTURAL ROLE
//! WorkspaceState is part of the **state organization layer**, representing
//! all workspace-specific state in the application.
//!
//! ## KEY COMPONENTS
//! - State: Main struct containing workspace-related fields
//!
//! ## ERROR HANDLING
//! Uses `Arc<Mutex<...>>` for thread-safe access with proper error handling.
//!
//! ## LOGGING
//! State changes are logged at appropriate levels.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly
//! - Avoid nested locks
//! - Use Arc for shared ownership
//!
//! ## TODO
//! - [ ] Add workspace validation
//! - [ ] Implement workspace change events
//! - [ ] Add workspace metrics

pub mod WorkspaceDelta;
pub mod WorkspaceState;

pub use WorkspaceState::*;
