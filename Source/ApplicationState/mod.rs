//! # ApplicationState
//!
//! Thread-safe state machine shared across all Mountain subsystems.
//! Organized into three tiers: DTOs (serializable data transfer objects),
//! Internal services (persistence, recovery, scanning), and State
//! (extension, feature, UI, and workspace state).
//!
//! ## Structure
//!
//! The state is wrapped in Arc and mutated via poisoned-lock guard patterns.
//! Every subsystem operates through the same ApplicationState instance,
//! ensuring consistency.

/// Serializable data transfer objects used for state persistence.
pub mod DTO;

pub mod State;

/// Internal services: extension scanning, path resolution, persistence,
/// recovery, serialization, and text processing.
pub mod Internal;
