//! # Wind Advanced Sync Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides advanced synchronization features between Wind's
//! frontend state and Mountain's backend state, including document sync and UI
//! state sync.
//!
//! ## ARCHITECTURAL ROLE
//! This module is the synchronization layer that keeps Wind and Mountain state
//! in sync in real-time.
//!
//! ## KEY COMPONENTS
//!
//! - **Sync**: Main WindAdvancedSync orchestrator
//!
//! ## ERROR HANDLING
//! All operations return Result types with descriptive error messages.
//!
//! ## LOGGING
//! Info-level for sync events, debug for operations, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient change detection
//! - Batched updates for performance
//! - Conflict detection and resolution
//!
//! ## TODO
//! - Add merge conflict resolution UI
//! - Implement offline sync support
//! - Add sync history tracking
//! - Support partial document sync

// Re-export the original file for backward compatibility
pub use crate::Element::Mountain::Source::IPC::WindAdvancedSync as Sync;
