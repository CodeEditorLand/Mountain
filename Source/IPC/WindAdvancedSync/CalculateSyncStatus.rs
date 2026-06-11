//! Derive a `SyncStatus` summary (totals per sync state) from the tracked
//! document map.

use std::collections::HashMap;

use crate::IPC::WindAdvancedSync::{SyncState, SyncStatus, SynchronizedDocument};

pub(crate) fn Fn(documents:&HashMap<String, SynchronizedDocument>) -> SyncStatus {
	let total = documents.len() as u32;

	let synced = documents.values().filter(|d| d.sync_state == SyncState::Synced).count() as u32;

	let conflicted = documents.values().filter(|d| d.sync_state == SyncState::Conflicted).count() as u32;

	let offline = documents.values().filter(|d| d.sync_state == SyncState::Offline).count() as u32;

	SyncStatus {
		total_documents:total,

		synced_documents:synced,

		conflicted_documents:conflicted,

		offline_documents:offline,

		last_sync_duration_ms:0,
	}
}
