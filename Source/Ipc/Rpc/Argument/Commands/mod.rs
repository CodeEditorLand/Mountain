
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to command execution and management.

#![allow(non_snake_case, non_camel_case_types)]

mod ExecuteCommandArgument;
mod RegisterArgument;

pub use ExecuteCommandArgument::ExecuteCommandArgument;
pub use RegisterArgument::RegisterArgument;
