//! # MarkerState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages marker state including custom documents, status bar items, and SCM state.
//!
//! ## ARCHITECTURAL ROLE
//! MarkerState is part of the **FeatureState** module, representing
//! marker-related state including custom documents, status bar items, and SCM.
//!
//! ## KEY COMPONENTS
//! - MarkerState: Main struct containing marker-related state
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
//! - [ ] Add marker validation
//! - [ ] Implement marker events
//! - [ ] Add marker metrics

pub mod MarkerState;
