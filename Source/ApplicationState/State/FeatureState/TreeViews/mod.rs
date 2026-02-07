//! # TreeViewState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages tree view providers state including tree view metadata and content.
//!
//! ## ARCHITECTURAL ROLE
//! TreeViewState is part of the **FeatureState** module, representing
//! tree view providers state organized by tree view ID.
//!
//! ## KEY COMPONENTS
//! - TreeViewState: Main struct containing active tree views map
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
//! - [ ] Add tree view validation
//! - [ ] Implement tree view events
//! - [ ] Add tree view metrics

pub mod TreeViewState;
