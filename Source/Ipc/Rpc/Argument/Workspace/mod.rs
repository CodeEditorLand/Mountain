
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to workspace management and operations.

#![allow(non_snake_case, non_camel_case_types)]

mod FindFilesArgument;
mod FindFilesOptions;
mod ResolveFolderArgument;

pub use FindFilesArgument::FindFilesArgument;
pub use FindFilesOptions::FindFilesOptionsDto;
pub use ResolveFolderArgument::ResolveFolderArgument;
