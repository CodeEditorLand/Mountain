//! Spawn the background document-synchronization task: every 5 seconds,
//! recompute sync status for modified documents and optionally emit
//! `mountain_sync_status_update` (gated by `LAND_SYNC_STATUS_EMIT`).

use std::time::{Duration, SystemTime};

use tauri::Emitter;
use tokio::time::interval;

use crate::{
	IPC::WindAdvancedSync::{SyncState, WindAdvancedSync},
	dev_log,
};

pub(crate) async fn Fn(Sync:&WindAdvancedSync) {
	let document_sync = Sync.document_sync.clone();

	let runtime = Sync.runtime.clone();

	tokio::spawn(async move {
		let mut interval = interval(Duration::from_secs(5));

		loop {
			interval.tick().await;

			// Synchronize documents
			if let Ok(mut sync) = document_sync.lock() {
				let modified_docs:Vec<String> = sync
					.synchronized_documents
					.iter()
					.filter(|(_, document)| document.sync_state == SyncState::Modified)
					.map(|(doc_id, _)| doc_id.clone())
					.collect();

				if !modified_docs.is_empty() {
					dev_log!("ipc", "Synchronizing {} documents", modified_docs.len());

					// Simulate synchronization process
					sync.last_sync_time =
						SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;

					// Update sync status
					sync.sync_status = WindAdvancedSync::calculate_sync_status(&sync.synchronized_documents);

					// Emit sync event - off by default. The Sky
					// renderer has no subscriber for this channel;
					// every emit just queued behind keystrokes on
					// the shared Tauri IPC pipe. Set
					// `LAND_SYNC_STATUS_EMIT=1` to opt in for
					// debugging / future Sky consumers.
					if std::env::var("LAND_SYNC_STATUS_EMIT").is_ok() {
						let _ = runtime
							.Environment
							.ApplicationHandle
							.emit("mountain_sync_status_update", sync.sync_status.clone());
					}
				}
			}
		}
	});
}
