// @module sky_command (Handler)
// @description This module contains the logic for Tauri commands that are
// invoked directly by the Sky frontend UI. These commands typically handle
// window management, system information queries, and other direct UI-to-backend
// interactions that do not involve the `ActionEffect` system. Renamed from
// `SkyCommand`.
//

#![allow(non_snake_case)]

mod SkyCommandLogic;

pub use self::SkyCommandLogic::*;
