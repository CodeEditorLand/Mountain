

/**
 * @module documents (Handlers)
 * @description This module contains the core logic for all document-related operations,
 * including lifecycle management (open, save, apply changes) and sending state
 * notifications to the Cocoon sidecar.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod DocumentsLogic;
mod NotificationLogic;

pub use self::DocumentsLogic::*;
pub use self::NotificationLogic::*;
