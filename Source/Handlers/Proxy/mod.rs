// File: Handlers/Proxy/mod.rs
// This module defines and exports handlers for proxying requests,
// potentially acting as a passthrough to the Cocoon extension host.
// This is likely deprecated in favor of specific gRPC service calls.

#![allow(non_snake_case, non_camel_case_types)]

mod Proxy; // Contains the logic for the proxy passthrough handler

pub use self::Proxy::*; // Re-export all public functions from Proxy.rs
