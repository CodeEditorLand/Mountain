// @module fs (Handlers)
// @description This module contains the core logic for handling all native
// filesystem operations. It aggregates and exports the handler functions from
// its sub-modules.
//

#![allow(non_snake_case, non_camel_case_types)]

mod FsLogic;

pub use self::FsLogic::*;
