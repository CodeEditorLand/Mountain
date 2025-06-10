// @module sky_commands (Handlers)
// @description This module contains the logic for Tauri commands that are
// invoked directly by the Sky frontend UI. These commands typically handle
// window management, system information queries, and other direct UI-to-backend
// interactions that do not involve the `ActionEffect` system.
//

#![allow(non_snake_case, non_camel_case_types)]

mod SkyCommandsLogic;

pub use self::SkyCommandsLogic::*;
