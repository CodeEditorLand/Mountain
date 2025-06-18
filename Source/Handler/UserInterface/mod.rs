// @module ui (Handler)
// @description This module contains the core logic for handling User Interface interactions
// like dialogs, messages, and quick picks. It aggregates and exports the
// handler functions from its sub-modules. Renamed from `UserInterface`.
//

#![allow(non_snake_case)]

mod UiLogic;

pub use self::UiLogic::*;
