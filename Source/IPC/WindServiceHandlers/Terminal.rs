#![allow(non_snake_case)]

//! # Terminal handlers
//!
//! Two related responsibilities:
//!
//! - `Terminal*` - operate on PTYs that are already registered via the
//!   `TerminalProvider` trait. Every method takes a provider-assigned
//!   `terminal_id` (`u64`).
//! - `LocalPTY*` - read-only platform queries that don't touch any registered
//!   PTY: discover available shells, default shell, and the current process
//!   environment.
//!
//! Layout (one export per file, file name = identity):
//! - `TerminalCreate::TerminalCreate`, `TerminalSendText::TerminalSendText`,
//!   `TerminalDispose::TerminalDispose`, `TerminalShow::TerminalShow`,
//!   `TerminalHide::TerminalHide`.
//! - `LocalPTYGetProfiles::LocalPTYGetProfiles`,
//!   `LocalPTYGetDefaultShell::LocalPTYGetDefaultShell`,
//!   `LocalPTYGetEnvironment::LocalPTYGetEnvironment`.

pub mod LocalPTYGetDefaultShell;

pub mod LocalPTYGetEnvironment;

pub mod LocalPTYGetProfiles;

pub mod TerminalCreate;

pub mod TerminalDispose;

pub mod TerminalHide;

pub mod TerminalSendText;

pub mod TerminalShow;
