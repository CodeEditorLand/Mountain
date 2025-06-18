// @module process_management (Handler)
// @description This module contains the core logic for managing the lifecycle
// of external sidecar processes, specifically the `Cocoon` extension host. It
// handles spawning the process, establishing communication, and performing the
// initial handshake.
//

#![allow(non_snake_case)]

mod CocoonManagement;
mod InitializationData;

pub use self::CocoonManagement::*;
// InitializationData contains only helper functions and does not need to be publicly
// re-exported from the handler root.
