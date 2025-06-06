
// This module defines the argument structures (DTOs) used for
// RPC calls related to managing status bar entries.

mod DisposeEntryArgument; // Renamed from Disposeentryargument
mod SetEntryArgument; // Renamed from Setentryargument

pub use DisposeEntryArgument::DisposeEntryArgument;
pub use SetEntryArgument::SetEntryArgument;
