// File: Rpc/Argument/Output/mod.rs
// This module defines the argument structures (DTOs) used for
// RPC calls related to managing output channels.

mod AppendArgument; // Renamed from Appendargument
mod IdArgument; // Renamed from Idargument (used by Clear, Close, Dispose)
mod RegisterArgument; // Renamed from Registerargument
mod ReplaceArgument; // Renamed from Replaceargument
mod RevealArgument; // Renamed from Revealargument

pub use AppendArgument::AppendArgument;
pub use IdArgument::IdArgument as OutputChannelIdentifierArgument; // Aliased for clarity
pub use RegisterArgument::RegisterArgument as RegisterOutputChannelArgument; // Aliased
pub use ReplaceArgument::ReplaceArgument;
pub use RevealArgument::RevealArgument;
