//! Force the file sink to initialise before the first
//! `dev_log!` so a panic on the boot path still produces a
//! header line + opt-in path. Harmless to call multiple times.

use crate::IPC::DevLog::WriteToFile;

pub fn Fn() { let _ = WriteToFile::InitFileSink(); }
