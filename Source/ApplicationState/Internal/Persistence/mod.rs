//! # Persistence
//!
//! Memento loading and saving for state persistence and crash recovery.
//! Writes/reads JSON state files to disk with proper error handling.

/// Mementoloader module.
pub mod MementoLoader;

/// Mementosaver module.
pub mod MementoSaver;
