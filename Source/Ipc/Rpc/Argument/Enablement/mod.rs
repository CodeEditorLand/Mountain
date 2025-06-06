
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to extension enablement state management.

#![allow(non_snake_case, non_camel_case_types)]

mod GetEnablementStateArgument;
mod SetEnablementArgument;

pub use GetEnablementStateArgument::GetEnablementStateArgument;
pub use SetEnablementArgument::SetEnablementArgument;
