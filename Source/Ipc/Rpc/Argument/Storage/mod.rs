
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to Memento storage (global and workspace).

#![allow(non_snake_case, non_camel_case_types)]

mod GetValueArgument;
mod SetValueArgument;

pub use GetValueArgument::{GetValueArgument, TargetDto};
pub use SetValueArgument::SetValueArgument;
