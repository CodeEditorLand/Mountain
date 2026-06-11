//! Document synchronization loop with a circuit-breaker: applies pending
//! changes every 5 seconds, slowing to a 30-second interval after repeated
//! consecutive failures and restoring on success.

use std::time::Duration;

use tokio::time::interval;

use crate::{IPC::WindAdvancedSync::WindAdvancedSync, dev_log};

pub(crate) async fn Fn(Sync:&WindAdvancedSync) {
	let mut interval = interval(Duration::from_secs(5));

	let mut consecutive_failures = 0;

	let max_consecutive_failures = 3;

	loop {
		interval.tick().await;

		dev_log!("lifecycle", "Synchronizing documents");

		// ERROR RECOVERY: Microsoft-inspired circuit breaker pattern
		let sync_start = std::time::Instant::now();

		let mut success_count = 0;

		let mut error_count = 0;

		// Get document changes from Wind
		let changes = Sync.get_pending_changes().await;

		// Apply changes to Mountain
		for change in changes {
			match Sync.apply_document_change(change).await {
				Ok(_) => success_count += 1,

				Err(e) => {
					error_count += 1;

					dev_log!("ipc", "error: [WindAdvancedSync] Failed to apply document change: {}", e);

					// ERROR HANDLING: Exponential backoff on consecutive failures
					consecutive_failures += 1;

					if consecutive_failures >= max_consecutive_failures {
						dev_log!("lifecycle", "Too many consecutive failures, slowing sync interval");

						// Reduce sync frequency to 30-second interval to prevent system overload
						// during persistent error conditions (circuit breaker pattern).
						interval = tokio::time::interval(Duration::from_secs(30));
					}
				},
			}
		}

		// Reset failure counter on successful operations
		if success_count > 0 {
			consecutive_failures = 0;

			// Restore normal sync frequency to 5-second interval after successful recovery.
			interval = tokio::time::interval(Duration::from_secs(5));
		}

		// Update sync status
		Sync.update_sync_status().await;

		// PERFORMANCE MONITORING: Microsoft-inspired metrics collection
		let sync_duration = sync_start.elapsed();

		dev_log!(
			"ipc",
			"[WindAdvancedSync] Document sync completed: {} success, {} errors, {:.2}ms",
			success_count,
			error_count,
			sync_duration.as_millis()
		);
	}
}
