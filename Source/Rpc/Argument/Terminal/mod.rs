// File: Rpc/Argument/Terminal/mod.rs
// This module defines the argument structures (DTOs) used for
// RPC calls related to terminal management.

mod CreateTerminalArgument; // Renamed from Createterminalargument
mod IdArgument; // Renamed from Idargument (used by Hide, Dispose)
mod SendTextArgument; // Renamed from Sendtextargument
mod ShowArgument; // Renamed from Showargument

pub use CreateTerminalArgument::CreateTerminalArgument;
pub use IdArgument::IdArgument as TerminalIdentifierArgument; // Aliased for clarity
pub use SendTextArgument::SendTextArgument;
pub use ShowArgument::ShowArgument;
