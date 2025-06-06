
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to window management and interactions.

#![allow(non_snake_case, non_camel_case_types)]

mod AsExternalUriArgument;
mod OpenUriArgument;

pub use AsExternalUriArgument::{AsExternalUriArgument, OptionsDto as AsExternalUriOptionsDto};
pub use OpenUriArgument::{OpenUriArgument, OptionsDto as OpenUriOptionsDto};
