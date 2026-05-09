#![allow(non_snake_case)]

//! DNS server startup-time storage. The wall-clock instant the
//! Hickory server bound its UDP socket is captured once and
//! returned to the webview via `dns_get_server_info`.
//!
//! Two siblings live here for cohesion: `init_dns_startup_time`
//! (fire-and-forget setter called from the bind path) and
//! private `Get` (read accessor used by `dns_get_server_info`).

use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::OnceCell;

static DNS_STARTUP_TIME:OnceCell<String> = OnceCell::new();

/// Records the moment the DNS server starts. Idempotent - the
/// `OnceCell` swallows subsequent calls.
pub fn init_dns_startup_time() {

	let now_iso = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|d| {
			let secs = d.as_secs();
			let hh = (secs % 86400) / 3600;
			let mm = (secs % 3600) / 60;
			let ss = secs % 60;
			format!("T{:02}:{:02}:{:02}Z", hh, mm, ss)
		})
		.unwrap_or_else(|_| "unknown".to_string());

	let _ = DNS_STARTUP_TIME.set(now_iso);
}

pub(super) fn Get() -> String { DNS_STARTUP_TIME.get().cloned().unwrap_or_else(|| "unknown".to_string()) }
