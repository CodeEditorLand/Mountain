//! 100ms real-time broadcast loop. No-ops (without draining the queue)
//! while no subscribers are registered so the shared Tauri IPC channel
//! stays free for keystrokes during extension boot.

use std::time::Duration;

use tokio::time::interval;

use crate::{IPC::WindAdvancedSync::WindAdvancedSync, dev_log};

pub(crate) async fn Fn(Sync:&WindAdvancedSync) {
	let mut interval = interval(Duration::from_millis(100));

	loop {
		interval.tick().await;

		// Fast-path: when no subscribers are registered the queue
		// can never reach a consumer. Skip the lock-and-drain path
		// entirely so the 100ms tick is a true no-op until Sky
		// registers a subscriber. This keeps the shared Tauri IPC
		// channel free for keystrokes during extension boot.
		{
			let rt = Sync.real_time_updates.lock().unwrap_or_else(|e| e.into_inner());

			if rt.Subscribers.is_empty() {
				continue;
			}
		}

		let updates = Sync.get_pending_updates().await;

		if !updates.is_empty() {
			// Broadcast updates to subscribers
			if let Err(e) = Sync.broadcast_updates(updates).await {
				dev_log!("ipc", "error: [WindAdvancedSync] Failed to broadcast updates: {}", e);
			}
		}
	}
}
