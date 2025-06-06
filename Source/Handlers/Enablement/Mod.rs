// File: Handlers/Enablement/Mod.rs
// This module defines and exports handlers for managing the enablement state of
// extensions. It determines if an extension is enabled, disabled, or enabled
// only for the current workspace.

mod Enablement; // Contains the primary logic for handling enablement state

pub use self::Enablement::*; // Re-export all public items from Enablement.rs
