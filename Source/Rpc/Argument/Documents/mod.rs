
// This module defines the argument structures (DTOs) used for
// RPC calls related to document management (opening, saving, etc.).

mod SaveAllArgument; // Renamed from Saveallargument
mod TryOpenArgument; // Renamed from Tryopenargument
mod TrySaveArgument; // Renamed from Trysaveargument
mod TrySaveAsArgument; // Renamed from Trysaveasargument

pub use SaveAllArgument::SaveAllArgument;
pub use TryOpenArgument::TryOpenArgument;
pub use TrySaveArgument::TrySaveArgument;
pub use TrySaveAsArgument::TrySaveAsArgument;
