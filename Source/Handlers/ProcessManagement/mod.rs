// File: Handlers/ProcessManagement/Mod.rs
// This module defines and exports handlers for managing sidecar processes,
// specifically the Cocoon extension host.

#![allow(non_snake_case, non_camel_case_types)]

mod ProcessManagement; // Contains logic for launching and managing sidecar processes

pub use self::ProcessManagement::*; // Re-export all public functions from ProcessManagement.rs
