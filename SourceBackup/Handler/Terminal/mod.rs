// @module terminal (Handler)
// @description This module contains the core logic for managing integrated
// terminal instances. It handles the creation of pseudo-terminals (PTYs),
// manages their I/O streams, and communicates their state to the extension host
// and User Interface.
//

#![allow(non_snake_case)]

mod TerminalLogic;

pub use self::TerminalLogic::*;
