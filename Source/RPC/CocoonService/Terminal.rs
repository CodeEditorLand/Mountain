#![allow(non_snake_case)]

//! Terminal-domain handlers for `CocoonService`. Eight entry points cover
//! lifecycle (open/close), I/O (input/data), notifications (opened/closed/
//! processId), and resize.

pub mod AcceptTerminalClosed;

pub mod AcceptTerminalOpened;

pub mod AcceptTerminalProcessData;

pub mod AcceptTerminalProcessId;

pub mod CloseTerminal;

pub mod OpenTerminal;

pub mod ResizeTerminal;

pub mod TerminalInput;
