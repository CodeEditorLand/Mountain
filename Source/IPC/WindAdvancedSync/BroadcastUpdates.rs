//! Emit each queued real-time update to every subscriber registered for its
//! target via the `real-time-update-{subscriber}` Tauri event channel.

use tauri::Emitter;

use crate::{
	IPC::WindAdvancedSync::{RealTimeUpdate, WindAdvancedSync},
	dev_log,
};

pub(crate) async fn Fn(Sync:&WindAdvancedSync, updates:Vec<RealTimeUpdate>) -> Result<(), String> {
	for update in updates {
		// Get subscribers for this target
		let subscribers = {
			let rt = Sync.real_time_updates.lock().unwrap_or_else(|e| e.into_inner());

			rt.Subscribers.get(&update.target).cloned()
		};

		// Broadcast to all subscribers for this target
		if let Some(subscriber_list) = subscribers {
			for subscriber in subscriber_list {
				if let Err(e) = Sync
					.runtime
					.Environment
					.ApplicationHandle
					.emit(&format!("real-time-update-{}", subscriber), &update)
				{
					dev_log!("ipc", "error: [WindAdvancedSync] Failed to broadcast to {}: {}", subscriber, e);
				}
			}
		}
	}

	Ok(())
}
