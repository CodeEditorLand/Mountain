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
//! - `SerializeTerminalState::SerializeTerminalState` - snapshot all active
//!   terminals to `ISerializedTerminalState[]` for window-reload restoration.
//! - `ReviveTerminalProcesses::ReviveTerminalProcesses` - respawn terminals
//!   from a previously serialised snapshot.
//! - `AttachToProcess::AttachToProcess` - reconnect the workbench to an
//!   existing Mountain PTY after a window reload.
//! - `DetachFromProcess::DetachFromProcess` - detach the workbench; PTY stays
//!   alive with output buffering for the next attach.

pub mod AttachToProcess;

pub mod LocalPTYCreateProcess;

pub mod LocalPTYFreePortKillProcess;

pub mod LocalPTYResize;

pub mod DetachFromProcess;

pub mod LocalPTYGetDefaultShell;

pub mod LocalPTYGetEnvironment;

pub mod LocalPTYGetProfiles;

pub mod ReviveTerminalProcesses;

pub mod SerializeTerminalState;

pub mod TerminalCreate;

pub mod TerminalDispose;

pub mod TerminalHide;

pub mod TerminalSendText;

pub mod TerminalShow;

pub mod TerminalRouter;
