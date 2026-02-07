//! # TerminalState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages terminal instances state including terminal metadata and content.
//!
//! ## ARCHITECTURAL ROLE
//! TerminalState is part of the **FeatureState** module, representing
//! terminal instances state organized by terminal ID.
//!
//! ## KEY COMPONENTS
//! - TerminalState: Main struct containing active terminals map
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
//! - [ ] Add terminal validation
//! - [ ] Implement terminal events
//! - [ ] Add terminal metrics

pub mod TerminalState;
