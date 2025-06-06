
// This module defines the argument structures (DTOs) used for
// RPC calls related to command execution and management.

mod ExecuteCommandArgument; // Renamed from Executecommandargument
mod RegisterArgument; // Renamed from Registerargument

pub use ExecuteCommandArgument::ExecuteCommandArgument;
pub use RegisterArgument::RegisterArgument;
