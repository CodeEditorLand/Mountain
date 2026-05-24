//! # Output channel handlers
//!
//! `vscode.window.createOutputChannel(...)` flows in from Cocoon
//! via gRPC; the WindServiceHandlers dispatcher (`mod.rs:648-663`)
//! invokes one of these five free functions which emit a
//! `sky://output/*` Tauri event for the renderer to mount /
//! mutate / focus the channel panel.
//!
//! Layout (one export per file, file name = identity):
//! - `OutputCreate::Fn`
//! - `OutputAppend::Fn`
//! - `OutputAppendLine::Fn`
//! - `OutputClear::Fn`
//! - `OutputShow::Fn`

pub mod OutputAppend;

pub mod OutputAppendLine;

pub mod OutputClear;

pub mod OutputCreate;

pub mod OutputShow;
