// File: Rpc/Argument/Diagnostics/mod.rs
// This module defines the argument structures (DTOs) used for
// RPC calls related to diagnostics management (e.g., problems, errors,
// warnings).

mod ChangeManyArgument; // Renamed from Changemanyargument
mod GetDiagnosticsArgument; // Renamed from Getdiagnosticsargument

pub use ChangeManyArgument::ChangeManyArgument;
pub use GetDiagnosticsArgument::GetDiagnosticsArgument;
