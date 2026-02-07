//! # ExtensionState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages extension-related state including extension registry, provider
//! registration, and scanned extensions.
//!
//! ## ARCHITECTURAL ROLE
//! ExtensionState is part of the **state organization layer**, representing
//! all extension-related state in the application.
//!
//! ## KEY COMPONENTS
//! - ExtensionRegistry: Command registry and provider handle management
//! - ProviderRegistration: Language providers registration
//! - ScannedExtensions: Discovered extensions metadata
//! - State: Main struct combining all extension state
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
//! - [ ] Implement extension lifecycle events
//! - [ ] Add extension metrics

pub mod ExtensionRegistry;
pub mod ProviderRegistration;
pub mod ScannedExtensions;
pub mod State;

pub use ExtensionRegistry::*;
pub use ProviderRegistration::*;
pub use ScannedExtensions::*;
