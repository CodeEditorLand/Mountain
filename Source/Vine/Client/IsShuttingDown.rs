
//! Whether the Vine client has been marked shutting down.

use crate::Vine::Client::Shared;

pub fn Fn() -> bool { Shared::ShutdownFlagLoad() }
