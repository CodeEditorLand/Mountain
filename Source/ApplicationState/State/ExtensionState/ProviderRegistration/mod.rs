//! # ProviderRegistration Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages language providers registration state.
//!
//! ## ARCHITECTURAL ROLE
//! ProviderRegistration is part of the **ExtensionState** module, representing
//! language provider registration state.
//!
//! ## KEY COMPONENTS
//! - Registration: Main struct containing language providers map
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
//! - [ ] Add provider validation
//! - [ ] Implement provider lifecycle events
//! - [ ] Add provider metrics

pub mod ProviderRegistration;

pub use ProviderRegistration::*;
