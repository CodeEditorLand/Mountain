// @module sky_IPC_bridge (Handler)
// @description This module contains the logic for bridging generic IPC messages
// from the Sky frontend directly to the Cocoon sidecar. This is used for parts
// of the VS Code workbench UI that expect to communicate via a generic
// `IPCRenderer.send/invoke` mechanism.
//

#![allow(non_snake_case)]

mod SkyIPCBridgeLogic;

pub use self::SkyIPCBridgeLogic::*;
