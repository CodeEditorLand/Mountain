// @module command (Handler)
// @description This module contains the core logic for managing the command
// registry and dispatching command execution. It aggregates and exports the
// handler functions and related types from its sub-modules.
//

#![allow(non_snake_case)]

mod CommandHandler;
mod CommandLogic;

pub use self::{CommandHandler::CommandHandler, CommandLogic::*};
