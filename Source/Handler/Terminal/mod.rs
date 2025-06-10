// @module terminal (Handlers)
// @description This module contains the core logic for managing integrated
// terminal instances. It handles the creation of pseudo-terminals (PTYs),
// manages their I/O streams, and communicates their state to the extension host
// and UI.
//

#![allow(non_snake_case, non_camel_case_types)]

mod TerminalLogic;

pub use self::TerminalLogic::*;
