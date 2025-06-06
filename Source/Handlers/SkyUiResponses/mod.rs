// File: Handlers/SkyUiResponses/mod.rs
// This module defines and exports handlers that process responses from the Sky
// (frontend) for UI interactions that were initiated by the Mountain backend,
// such as dialog or quick pick selections.

#![allow(non_snake_case, non_camel_case_types)]

mod SkyUiResponses; // Contains the logic for handling UI response callbacks

pub use self::SkyUiResponses::*; // Re-export all public functions from SkyUiResponses.rs
