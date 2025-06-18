// @module secret (Handler)
// @description This module contains the core logic for handling secure secret
// storage operations by interacting with the native OS keychain via the
// `keyring` crate. Renamed from `secrets` for consistency.
//

#![allow(non_snake_case)]

mod SecretsLogic;

pub use self::SecretsLogic::*;
