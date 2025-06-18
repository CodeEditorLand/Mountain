// @module status_bar (Handler)
// @description This module contains the core logic for managing status bar
// items. It handles RPC calls from Cocoon to create, update, and remove items,
// and emits events to the Sky frontend to render the changes.
//

#![allow(non_snake_case)]

mod StatusBarLogic;

pub use self::StatusBarLogic::*;
