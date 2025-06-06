// File: Handlers/Protocol/Mod.rs
// This module defines and exports handlers for custom URI scheme requests,
// such as `vscode://`, routing them to appropriate actions within the
// application.

#![allow(non_snake_case, non_camel_case_types)]

mod Protocol; // Contains the logic for handling custom URI schemes

pub use self::Protocol::*; // Re-export all public functions from Protocol.rs
