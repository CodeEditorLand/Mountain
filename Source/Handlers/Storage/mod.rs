// File: Handlers/Storage/mod.rs
// This module defines and exports handlers for managing Memento storage,
// including both global and workspace-scoped data persistence.

#![allow(non_snake_case, non_camel_case_types)]

mod Storage; // Contains the logic for getting and setting storage values

pub use self::Storage::*; // Re-export all public functions from Storage.rs
