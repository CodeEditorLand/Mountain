//! # ConfigurationState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages configuration and storage state including global configuration,
//! workspace configuration, and memento storage buffers.
//!
//! ## ARCHITECTURAL ROLE
//! ConfigurationState is part of the **state organization layer**, representing
//! all configuration and storage-related state in the application.
//!
//! ## KEY COMPONENTS
//! - State: Main struct containing configuration and storage fields
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
//! - [ ] Add configuration validation
//! - [ ] Implement configuration change events
//! - [ ] Add configuration diffing

pub mod ConfigurationState;

pub use ConfigurationState::*;
