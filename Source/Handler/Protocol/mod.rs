// @module protocol (Handler)
// @description This module contains the logic for handling custom URI scheme
// requests, such as `vscode://file/...`. It is registered with Tauri in
// `Binary.rs` and is responsible for parsing the request and dispatching it to
// the appropriate action.
//

#![allow(non_snake_case)]

mod ProtocolLogic;

pub use self::ProtocolLogic::*;
