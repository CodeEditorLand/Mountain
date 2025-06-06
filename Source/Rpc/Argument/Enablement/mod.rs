
// This module defines the argument structures (DTOs) used for
// RPC calls related to extension enablement state management.

mod GetEnablementStateArgument; // Renamed from Getenablementstateargument
mod SetEnablementArgument; // Renamed from Setenablementargument

pub use GetEnablementStateArgument::GetEnablementStateArgument;
pub use SetEnablementArgument::SetEnablementArgument;
