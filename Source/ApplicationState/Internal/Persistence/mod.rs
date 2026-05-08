//! # Persistence Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Handles memento loading and saving for state persistence and crash recovery.
//!
//! ## ARCHITECTURAL ROLE
//! Persistence is part of the **Internal** module, providing file I/O
//! operations for state persistence.
//!
//! ## KEY COMPONENTS
//! - MementoLoader: Loads memento state from disk
//! - MementoSaver: Saves memento state to disk
//!
//! ## ERROR HANDLING
//! Functions return Result<T, CommonError> with comprehensive error handling.
//!
//! ## LOGGING
//! All operations are logged at appropriate levels.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Synchronous file I/O for initialization
//! - Proper error handling and recovery
//!
//! ## TODO
//! - [ ] Add checksum validation
//! - [ ] Implement incremental updates
//! - [ ] Add compression support

pub mod MementoLoader;

pub mod MementoSaver;
