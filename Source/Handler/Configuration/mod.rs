// @module config (Handler)
// @description This module contains the core logic for configuration
// management, including reading, merging, updating, and inspecting settings
// from various `settings.json` files.
//

#![allow(non_snake_case)]

mod ConfigurationLogic;

pub use self::ConfigurationLogic::*;
