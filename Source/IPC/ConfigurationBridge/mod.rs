//! Bidirectional configuration synchronization between Mountain's Rust backend
//! and Wind's TypeScript frontend.
//!
//! Re-exports types from the parent `ConfigurationBridge.rs` for
//! backward compatibility. The actual `ConfigurationBridge` struct and its
//! methods live one level up.
//!
//! ## Planned Work
//!
//! In a future refactoring, split `ConfigurationBridge.rs` into atomic
//! structure definitions and move those into `Bridge.rs` within this
//! directory, leaving this `mod.rs` as a clean re-export layer.

// Re-export the original ConfigurationBridge types for backward compatibility
