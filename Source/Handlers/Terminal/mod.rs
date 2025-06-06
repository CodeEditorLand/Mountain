// File: Handlers/Terminal/mod.rs
// This module defines and exports handlers for managing terminal instances,
// including creation, interaction, and lifecycle events.

#![allow(non_snake_case, non_camel_case_types)]

mod Terminal; // Contains the logic for handling terminal operations

pub use self::Terminal::*; // Re-export all public functions from Terminal.rs
