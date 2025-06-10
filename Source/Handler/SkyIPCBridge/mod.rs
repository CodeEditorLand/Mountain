// @module sky_ipc_bridge (Handlers)
// @description This module contains the logic for bridging generic IPC messages
// from the Sky frontend directly to the Cocoon sidecar. This is used for parts
// of the VS Code workbench UI that expect to communicate via a generic
// `ipcRenderer.send/invoke` mechanism.
//

#![allow(non_snake_case, non_camel_case_types)]

mod SkyIpcBridgeLogic;

pub use self::SkyIpcBridgeLogic::*;
