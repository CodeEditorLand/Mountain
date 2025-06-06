// File: Ipc/Rpc/Argument/Output/mod.rs
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to managing output channels.

#![allow(non_snake_case, non_camel_case_types)]

mod AppendArgument;
mod IdArgument;
mod RegisterArgument;
mod ReplaceArgument;
mod RevealArgument;

pub use AppendArgument::AppendArgument;
pub use IdArgument::IdArgument as OutputChannelIdentifierArgument; // Aliased for clarity
pub use RegisterArgument::RegisterArgument as RegisterOutputChannelArgument; // Aliased
pub use ReplaceArgument::ReplaceArgument;
pub use RevealArgument::RevealArgument;
