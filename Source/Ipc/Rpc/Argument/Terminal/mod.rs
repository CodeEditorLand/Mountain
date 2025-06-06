// File: Ipc/Rpc/Argument/Terminal/mod.rs
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to terminal management.

#![allow(non_snake_case, non_camel_case_types)]

mod CreateTerminalArgument;
mod IdArgument;
mod SendTextArgument;
mod ShowArgument;

pub use CreateTerminalArgument::CreateTerminalArgument;
pub use IdArgument::IdArgument as TerminalIdentifierArgument; // Aliased for clarity
pub use SendTextArgument::SendTextArgument;
pub use ShowArgument::ShowArgument;
