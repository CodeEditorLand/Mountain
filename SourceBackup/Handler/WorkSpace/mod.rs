// @module workspace (Handler)
// @description This module contains the core logic for handling
// workspace-related operations, such as querying folders and finding files. It
// aggregates and exports the handler functions from its sub-modules.
// Renamed from `WorkSpace`.

#![allow(non_snake_case)]

mod WorkSpaceLogic;

pub use self::WorkSpaceLogic::*;
