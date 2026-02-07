//! # ScannedExtensions Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages scanned extensions metadata state.
//!
//! ## ARCHITECTURAL ROLE
//! ScannedExtensions is part of the **ExtensionState** module, representing
//! discovered extensions metadata state.
//!
//! ## KEY COMPONENTS
//! - Extensions: Main struct containing scanned extensions map
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
//! - [ ] Add extension validation
//! - [ ] Implement extension discovery events
//! - [ ] Add extension metrics

pub mod ScannedExtensions;

pub use ScannedExtensions::*;
