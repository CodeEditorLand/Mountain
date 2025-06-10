

/**
 * @module extension_management (Handlers)
 * @description This module contains the core logic for managing extensions. This
 * includes scanning the filesystem for installed extensions and managing their
 * enablement state (enabled/disabled).
 */

#![allow(non_snake_case, non_camel_case_types)]

mod EnablementLogic;
mod ScanLogic;

pub use self::EnablementLogic::*;
pub use self::ScanLogic::*;
