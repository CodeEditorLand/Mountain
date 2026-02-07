//! # Internal Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Contains internal utility functions for the ApplicationState module,
//! handling tasks like file I/O, path resolution, and serialization.
//!
//! ## ARCHITECTURAL ROLE
//! The Internal module provides **private utilities** for ApplicationState:
//!
//! ```text
//! ApplicationState (Public API) ──► Internal (Private Utilities)
//!                                       │
//!                                       ├── Persistence
//!                                       ├── PathResolution
//!                                       ├── Serialization
//!                                       ├── ExtensionScanner
//!                                       ├── TextProcessing
//!                                       └── Recovery
//! ```
//!
//! ## KEY COMPONENTS
//! - Persistence: Memento loading/saving utilities
//! - PathResolution: Path resolution utilities
//! - Serialization: URL serialization helpers
//! - ExtensionScanner: Extension scanning utilities
//! - TextProcessing: Text analysis utilities
//! - Recovery: Recovery utilities
//!
//! ## ERROR HANDLING
//! Functions return Result<T, CommonError> or provide default values with
//! logging for failures.
//!
//! ## LOGGING
//! All operations are logged at appropriate levels.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Synchronous file I/O for initialization
//! - Async operations where appropriate
//! - Proper error handling and recovery
//!
//! ## TODO
//! - [ ] Add checksum validation for memento files
//! - [ ] Implement incremental memento updates
//! - [ ] Add concurrent extension scanning

pub mod Persistence;
pub mod PathResolution;
pub mod Serialization;
pub mod ExtensionScanner;
pub mod TextProcessing;
pub mod Recovery;

pub use Persistence::*;
pub use PathResolution::*;
pub use Serialization::*;
pub use ExtensionScanner::*;
pub use TextProcessing::*;
pub use Recovery::*;
