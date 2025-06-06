// File: Ipc/Rpc/Argument/Diagnostics/mod.rs
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to diagnostics management (e.g., problems, errors,
// warnings).

#![allow(non_snake_case, non_camel_case_types)]

mod ChangeManyArgument;
mod GetDiagnosticsArgument;

pub use ChangeManyArgument::ChangeManyArgument;
pub use GetDiagnosticsArgument::GetDiagnosticsArgument;
