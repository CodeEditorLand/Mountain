// @module fs (Handler)
// @description This module contains the core logic for handling all native
// filesystem operations. It aggregates and exports the handler functions from
// its sub-modules. It was renamed from `FileSystem` for brevity.
//

#![allow(non_snake_case)]

mod FileSystemLogic;

pub use self::FileSystemLogic::*;
