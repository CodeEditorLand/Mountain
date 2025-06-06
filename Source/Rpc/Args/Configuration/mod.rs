// File: Rpc/Args/Configuration/Mod.rs
// This module defines the argument structures (DTOs) used for
// RPC calls related to application configuration management.

mod GetConfigurationArgument; // Renamed from Getconfigurationargument
mod InspectArgument; // Renamed from Inspectargument
mod UpdateArgument; // Renamed from Updateargument

pub use GetConfigurationArgument::{GetConfigurationArgument, OverridesDto}; // OverridesDto is closely related
pub use InspectArgument::InspectArgument;
pub use UpdateArgument::UpdateArgument;
