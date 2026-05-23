
//! Generate a fresh UUID-v4 (simple form) for use as an Air request id.
//! Each Air RPC carries one of these so Mountain can correlate replies
//! with the originating call across log lines + traces.

use uuid::Uuid;

pub fn Fn() -> String { Uuid::new_v4().simple().to_string() }
