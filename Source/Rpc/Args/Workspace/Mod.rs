// File: Rpc/Args/Workspace/Mod.rs
// This module defines the argument structures (DTOs) used for
// RPC calls related to workspace management and operations.

mod FindFilesArgument; // Renamed from Findfilesargument
mod FindFilesOptions; // Renamed from Findfilesoptions
mod ResolveFolderArgument; // Renamed from Resolvefolderargument

pub use FindFilesArgument::FindFilesArgument;
pub use FindFilesOptions::FindFilesOptionsDto; // Assuming Dto was the primary export
pub use ResolveFolderArgument::ResolveFolderArgument;
