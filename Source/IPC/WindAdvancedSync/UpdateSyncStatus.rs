//! Recompute the aggregate `sync_status` counters from the tracked document
//! map and stamp `last_sync_time`.

use std::time::SystemTime;

use crate::IPC::WindAdvancedSync::{SyncState, WindAdvancedSync};

pub(crate) async fn Fn(Sync:&WindAdvancedSync) {
	let mut sync = Sync.document_sync.lock().unwrap_or_else(|e| e.into_inner());

	sync.sync_status.total_documents = sync.synchronized_documents.len() as u32;

	sync.sync_status.synced_documents = sync
		.synchronized_documents
		.values()
		.filter(|doc| matches!(doc.sync_state, SyncState::Synced))
		.count() as u32;

	sync.sync_status.conflicted_documents = sync
		.synchronized_documents
		.values()
		.filter(|doc| matches!(doc.sync_state, SyncState::Conflicted))
		.count() as u32;

	sync.sync_status.offline_documents = sync
		.synchronized_documents
		.values()
		.filter(|doc| matches!(doc.sync_state, SyncState::Offline))
		.count() as u32;

	sync.last_sync_time = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs();
}
