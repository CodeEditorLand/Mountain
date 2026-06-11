//! Spawn the background performance-monitoring task: every 10 seconds,
//! refresh uptime/last-update stats and optionally emit
//! `mountain_performance_update` (gated by `LAND_PERF_EMIT`).

use std::time::{Duration, SystemTime};

use tauri::Emitter;
use tokio::time::interval;

use crate::IPC::WindAdvancedSync::WindAdvancedSync;

pub(crate) async fn Fn(Sync:&WindAdvancedSync) {
	let performance_stats = Sync.performance_stats.clone();

	let runtime = Sync.runtime.clone();

	tokio::spawn(async move {
		let mut interval = interval(Duration::from_secs(10));

		loop {
			interval.tick().await;

			if let Ok(mut stats) = performance_stats.lock() {
				stats.last_update =
					SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;

				stats.connection_uptime += 10;

				// Emit performance update - off by default. Same
				// reasoning as `mountain_sync_status_update`: no
				// Sky subscriber, every emit cost shared channel
				// bandwidth. Set `LAND_PERF_EMIT=1` to opt in.
				if std::env::var("LAND_PERF_EMIT").is_ok() {
					let _ = runtime
						.Environment
						.ApplicationHandle
						.emit("mountain_performance_update", stats.clone());
				}
			}
		}
	});
}
