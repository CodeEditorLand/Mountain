// @module secrets (Handlers)
// @description This module contains the core logic for handling secure secret
// storage operations by interacting with the native OS keychain via the
// `keyring` crate.
//

#![allow(non_snake_case, non_camel_case_types)]

mod SecretsLogic;

pub use self::SecretsLogic::*;
