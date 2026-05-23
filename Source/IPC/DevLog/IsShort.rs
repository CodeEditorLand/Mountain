
//! `true` when `Trace=short` is set - enables path aliasing
//! and consecutive-duplicate compression in `dev_log!`.

use std::sync::OnceLock;

use crate::IPC::DevLog::IsEnabled;

static SHORT_MODE:OnceLock<bool> = OnceLock::new();

pub fn Fn() -> bool { *SHORT_MODE.get_or_init(|| IsEnabled::EnabledTags().iter().any(|T| T == "short")) }
