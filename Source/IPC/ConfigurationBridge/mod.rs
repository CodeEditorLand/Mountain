//! # Configuration Bridge Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides bidirectional configuration synchronization between
//! Mountain's Rust backend and Wind's TypeScript frontend.
//!
//! ## ARCHITECTURAL ROLE
//! This module is the synchronization layer that maintains configuration
//! consistency across the Wind-Mountain bridge.
//!
//! ## KEY COMPONENTS
//!
//! - **Bridge**: Main ConfigurationBridge orchestrator
//!
//! ## ERROR HANDLING
//! All operations return Result types with descriptive error messages.
//!
//! ## LOGGING
//! Info-level for sync events, debug for operations, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient conflict resolution
//! - Batched updates for performance
//! - Caching for frequently accessed config
//!
//! ## TODO
//! - Add three-way merge support
//! - Implement conflict UI in Wind
//! - Add configuration validation schemas
//! - Support configuration versioning

// Re-export the original ConfigurationBridge types for backward compatibility
// The actual implementation is in the parent directory ConfigurationBridge.rs
// TODO: In future refactoring, split ConfigurationBridge.rs into atomic structure
// and move that structure into Bridge.rs file within this directory.
