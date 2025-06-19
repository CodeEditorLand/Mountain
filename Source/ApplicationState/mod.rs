//! # ApplicationState Module
//!
//! This module defines the central, shared, thread-safe state for the entire
//! Mountain application. It includes the main `ApplicationState` struct, all
//! of its related Data Transfer Objects (DTOs), and internal helper functions
//! for state management.

#![allow(non_snake_case, non_camel_case_types)]

// --- Public Modules ---

/// Defines the main `ApplicationState` struct and its core implementation.
pub mod ApplicationState;

/// Defines all DTOs used to represent the various components of the
/// application state.
pub mod DTO;

// --- Internal Implementation ---

/// Contains internal helper functions for the `ApplicationState` module.
pub mod Internal;
