

/**
 * @module app_state
 * @description This module defines the central, shared, thread-safe state for the
 * entire Mountain application. It includes the main `AppState` struct, all of its
 * related Data Transfer Objects (DTOs), and internal helper functions.
 */

#![allow(non_snake_case, non_camel_case_types)]

// --- Public Modules ---

/// Defines the main `AppState` struct and its core implementation.
pub mod AppState;

/// Defines all DTOs used to represent the various components of the application state.
pub mod Dto;

// --- Internal Implementation ---

/// Contains internal helper functions for the `AppState` module.
mod Internal;
