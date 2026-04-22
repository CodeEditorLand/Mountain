#![allow(non_snake_case)]

//! Wind Service Handler - Atomic domain modules.
//!
//! Each submodule contains IPC handler functions for a specific domain.
//! The parent `WindServiceHandlers.rs` imports and dispatches to these.

// Domain submodules
pub mod Command;
pub mod Configuration;
pub mod Decoration;
pub mod Environment;
pub mod Extension;
pub mod FileSystem;
pub mod History;
pub mod Keybinding;
pub mod Label;
pub mod Lifecycle;
pub mod Model;
pub mod NativeHost;
pub mod Notification;
pub mod Output;
pub mod Progress;
pub mod QuickInput;
pub mod Search;
pub mod Storage;
pub mod Terminal;
pub mod TextFile;
pub mod Theme;
pub mod Workspace;
pub mod WorkingCopy;

// Re-export shared helpers and types from the parent module so submodules
// can `use super::` to reach them. These are defined in WindServiceHandlers.rs
// and forwarded here via the parent's `pub use`.

// Type aliases re-exported for submodule convenience
pub use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	ConfigurationTarget::ConfigurationTarget,
};

// Helper functions from WindServiceHandlers.rs - re-exported for submodule use
pub use super::WindServiceHandlers::extract_path_from_arg;
pub use super::WindServiceHandlers::metadata_to_istat;
