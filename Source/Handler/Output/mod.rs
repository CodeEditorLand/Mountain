// @module output (Handler)
// @description This module contains the core logic for handling output channel
// operations, such as registering channels and appending content. It aggregates
// and exports the handler functions from its sub-modules.
//

#![allow(non_snake_case)]

mod OutputLogic;

pub use self::OutputLogic::*;
