//! Memento loading and saving for state persistence and crash recovery.
//! Writes and reads JSON state files to disk with proper error handling.

/// Memento loading with recovery and initial-load logic.
pub mod MementoLoader;

/// Memento saving to disk.
pub mod MementoSaver;
