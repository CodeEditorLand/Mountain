//! # UIState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages user interface request state including pending UI interactions
//! such as dialogs, prompts, and other synchronous UI requests.
//!
//! ## ARCHITECTURAL ROLE
//! UIState is part of the **state organization layer**, representing
//! user interface request state.
//!
//! ## KEY COMPONENTS
//! - State: Main struct containing pending UI requests
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
//! - [ ] Add UI request validation
//! - [ ] Implement UI request timeout
//! - [ ] Add UI request metrics

pub mod UIState;

pub use UIState::*;
