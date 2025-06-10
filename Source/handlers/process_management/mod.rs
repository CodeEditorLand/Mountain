

/**
 * @module process_management (Handlers)
 * @description This module contains the core logic for managing the lifecycle of
 * external sidecar processes, specifically the `Cocoon` extension host. It handles
 * spawning the process, establishing communication, and performing the initial handshake.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod CocoonManagement;
mod InitData;

pub use self::CocoonManagement::*;
// InitData contains only helper functions and does not need to be publicly re-exported.
