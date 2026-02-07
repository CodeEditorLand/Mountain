//! # DocumentState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages open documents state including document metadata and content.
//!
//! ## ARCHITECTURAL ROLE
//! DocumentState is part of the **FeatureState** module, representing
//! open documents state.
//!
//! ## KEY COMPONENTS
//! - DocumentState: Main struct containing open documents map
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
//! - [ ] Add document validation
//! - [ ] Implement document events
//! - [ ] Add document metrics

pub mod DocumentState;
