//! CONFLICT DETECTION: Microsoft-inspired conflict resolution. Rejects a
//! change when the target document was modified within the last 10 seconds
//! or is already in conflicted state.

use std::time::SystemTime;

use crate::IPC::WindAdvancedSync::{DocumentChange, SyncState, WindAdvancedSync};

pub(crate) async fn Fn(Sync:&WindAdvancedSync, change:&DocumentChange) -> Result<(), String> {
	let sync = Sync.document_sync.lock().unwrap_or_else(|e| e.into_inner());

	// Check if document exists and has been modified since last sync
	if let Some(document) = sync.synchronized_documents.get(&change.document_id) {
		let current_time = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		// If document was modified recently (within last 10 seconds), potential
		// conflict
		if current_time - document.last_modified < 10 {
			return Err(format!(
				"Document {} was modified recently ({}s ago)",
				document.document_id,
				current_time - document.last_modified
			));
		}

		// Check sync state for conflicts
		if matches!(document.sync_state, SyncState::Conflicted) {
			return Err(format!("Document {} is in conflicted state", document.document_id));
		}
	}

	Ok(())
}
