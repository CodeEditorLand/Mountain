
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to document management (opening, saving, etc.).

#![allow(non_snake_case, non_camel_case_types)]

mod SaveAllArgument;
mod TryOpenArgument;
mod TrySaveArgument;
mod TrySaveAsArgument;

pub use SaveAllArgument::SaveAllArgument;
pub use TryOpenArgument::TryOpenArgument;
pub use TrySaveArgument::TrySaveArgument;
pub use TrySaveAsArgument::TrySaveAsArgument;
