//! # DiagnosticsState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages diagnostic errors state including markers by owner and resource.
//!
//! ## ARCHITECTURAL ROLE
//! DiagnosticsState is part of the **FeatureState** module, representing
//! diagnostic errors state.
//!
//! ## KEY COMPONENTS
//! - DiagnosticsState: Main struct containing diagnostics map
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
//! - [ ] Add diagnostics validation
//! - [ ] Implement diagnostics events
//! - [ ] Add diagnostics metrics

pub mod DiagnosticsState;
