

/**
 * @module config (Handlers)
 * @description This module contains the core logic for configuration management,
 * including reading, merging, updating, and inspecting settings from various
 * `settings.json` files.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod ConfigLogic;

pub use self::ConfigLogic::*;
