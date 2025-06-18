// @module extension_management (Handler)
// @description This module contains the core logic for managing extensions.
// This includes scanning the filesystem for installed extensions and managing
// their enablement state (enabled/disabled).
//

#![allow(non_snake_case)]

mod EnablementLogic;
mod ScanLogic;

pub use self::{EnablementLogic::*, ScanLogic::*};
