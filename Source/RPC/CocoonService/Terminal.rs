//! Terminal-domain handlers for `CocoonService`. Eight entry points cover
//! lifecycle (open/close), I/O (input/data), notifications (opened/closed/
//! processId), and resize.
/// AcceptTerminalClosed handler: notifies the environment that a terminal
/// closed.
pub mod AcceptTerminalClosed;

/// AcceptTerminalOpened handler: notifies the environment that a terminal
/// opened.
pub mod AcceptTerminalOpened;

/// AcceptTerminalProcessData handler: forwards process output from a terminal.
pub mod AcceptTerminalProcessData;

/// AcceptTerminalProcessId handler: notifies the environment of the terminal's
/// PID.
pub mod AcceptTerminalProcessId;

/// CloseTerminal handler: closes an open terminal.
pub mod CloseTerminal;

/// OpenTerminal handler: opens a new terminal emulator.
pub mod OpenTerminal;

/// ResizeTerminal handler: resizes an open terminal.
pub mod ResizeTerminal;

/// TerminalInput handler: sends input data to an open terminal.
pub mod TerminalInput;
