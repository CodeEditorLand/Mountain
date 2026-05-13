//! # Persistence
//!
//! Memento loading and saving for state persistence and crash recovery.
//! Writes/reads JSON state files to disk with proper error handling.

pub mod MementoLoader;

pub mod MementoSaver;
