// File: Handlers/Commands/Mod.rs
// This module defines and exports handlers related to command execution and
// registration.

mod CommandHandler; // Defines the CommandHandler enum
mod Commands; // Contains the main logic for handling command operations

pub use self::{CommandHandler::CommandHandler, Commands::*}; // Re-export all public items from Commands.rs
