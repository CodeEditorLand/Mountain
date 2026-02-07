//! # WebviewState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages webview panels state including webview metadata and content.
//!
//! ## ARCHITECTURAL ROLE
//! WebviewState is part of the **FeatureState** module, representing
//! webview panels state organized by webview ID.
//!
//! ## KEY COMPONENTS
//! - WebviewState: Main struct containing active webviews map
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
//! - [ ] Add webview validation
//! - [ ] Implement webview events
//! - [ ] Add webview metrics

pub mod WebviewState;
