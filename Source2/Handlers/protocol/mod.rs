

/**
 * @module protocol (Handlers)
 * @description This module contains the logic for handling custom URI scheme
 * requests, such as `vscode://file/...`. It is registered with Tauri in `main.rs`
 * and is responsible for parsing the request and dispatching it to the
 * appropriate action.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod ProtocolLogic;

pub use self::ProtocolLogic::*;
