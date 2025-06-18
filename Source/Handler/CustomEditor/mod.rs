// @module custom_editor (Handler)
// @description This module contains the core logic for managing custom editor
// providers and the lifecycle of custom documents. It aggregates and exports
// the handler functions from its sub-modules.
//

#![allow(non_snake_case)]

mod CustomEditorLogic;

pub use self::CustomEditorLogic::*;
