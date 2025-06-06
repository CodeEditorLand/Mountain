
// This module defines and exports the argument structures (DTOs) used for RPC
// calls related to language-specific features like provider registration and
// event emission.

#![allow(non_snake_case, non_camel_case_types)]

mod EmitProviderEventArgument;
mod RegisterProviderArgument;
mod UnregisterProviderArgument;

pub use EmitProviderEventArgument::EmitProviderEventArgument;
pub use RegisterProviderArgument::RegisterProviderArgument;
pub use UnregisterProviderArgument::UnregisterProviderArgument;
