// File: Handlers/SkyIpcBridge/mod.rs
// This module defines and exports handlers that act as a bridge for IPC
// messages from the Sky (frontend), forwarding them to the Cocoon sidecar.
// This is likely to be deprecated in favor of more direct communication
// patterns.

#![allow(non_snake_case, non_camel_case_types)]

mod SkyIpcBridge; // Contains the logic for the IPC bridge handlers

pub use self::SkyIpcBridge::*; // Re-export all public functions from SkyIpcBridge.rs
