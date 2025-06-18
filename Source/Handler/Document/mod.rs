// @module document (Handler)
// @description This module contains the core logic for all document-related
// operations, including lifecycle management (open, save, apply changes) and
// sending state notifications to the Cocoon sidecar.
//

#![allow(non_snake_case)]

mod DocumentLogic;
mod NotificationLogic;

pub use self::{DocumentLogic::*, NotificationLogic::*};
