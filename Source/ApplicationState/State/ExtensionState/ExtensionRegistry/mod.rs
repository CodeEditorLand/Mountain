//! # ExtensionRegistry Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages extension registry including command registry and provider handle
//! management.
//!
//! ## ARCHITECTURAL ROLE
//! ExtensionRegistry is part of the **ExtensionState** module, representing
//! the command registry and provider handle management.
//!
//! ## KEY COMPONENTS
//! - Registry: Main struct containing command registry and provider state
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
//! - [ ] Add command validation
//! - [ ] Implement command discovery
//! - [ ] Add command metrics

pub mod ExtensionRegistry;

pub use ExtensionRegistry::*;
