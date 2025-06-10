

/**
 * @module commands (Handlers)
 * @description This module contains the core logic for managing the command registry
 * and dispatching command execution. It aggregates and exports the handler
 * functions and related types from its sub-modules.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod CommandHandler;
mod CommandsLogic;

pub use self::CommandHandler::CommandHandler;
pub use self::CommandsLogic::*;
