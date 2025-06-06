// File: Ipc/Rpc/Argument/StatusBar/mod.rs
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to managing status bar entries.

#![allow(non_snake_case, non_camel_case_types)]

mod DisposeEntryArgument;
mod SetEntryArgument;

pub use DisposeEntryArgument::DisposeEntryArgument;
pub use SetEntryArgument::SetEntryArgument;
