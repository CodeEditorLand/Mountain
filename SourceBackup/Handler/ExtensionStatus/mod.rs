// @module extension_status (Handler)
// @description This module contains the logic for handling extension lifecycle
// status notifications sent from the Cocoon sidecar back to the Mountain host.
// This includes events like activation success or failure.
//

#![allow(non_snake_case)]

mod StatusNotificationLogic;

pub use self::StatusNotificationLogic::*;
