
// This module defines the argument structures (DTOs) used for
// RPC calls related to window management and interactions.

mod AsExternalUriArgument; // Renamed from Asexternaluriargument
mod OpenUriArgument; // Renamed from Openuriargument
// AsExternalUriOption and OpenUriOption were nested within their respective
// argument files. They will be re-exported from their parent modules if needed.

pub use AsExternalUriArgument::{AsExternalUriArgument, OptionsDto as AsExternalUriOptionsDto};
pub use OpenUriArgument::{OpenUriArgument, OptionsDto as OpenUriOptionsDto};
