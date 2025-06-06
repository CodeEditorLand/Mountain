
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to application configuration management.

#![allow(non_snake_case, non_camel_case_types)]

mod GetConfigurationArgument;
mod InspectArgument;
mod UpdateArgument;

pub use GetConfigurationArgument::{GetConfigurationArgument, OverridesDto};
pub use InspectArgument::InspectArgument;
pub use UpdateArgument::UpdateArgument;
