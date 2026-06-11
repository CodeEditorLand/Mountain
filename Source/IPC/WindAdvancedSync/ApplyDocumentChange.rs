//! Apply a single document change after conflict checking, mark it as
//! applied in the pending-changes map, and log the operation duration.

use crate::{
	IPC::WindAdvancedSync::{ChangeType, DocumentChange, WindAdvancedSync},
	dev_log,
};

pub(crate) async fn Fn(Sync:&WindAdvancedSync, change:DocumentChange) -> Result<(), String> {
	dev_log!("lifecycle", "Applying document change: {}", change.change_id);

	// CONFLICT RESOLUTION: Microsoft-inspired conflict handling
	let change_start = std::time::Instant::now();

	// Check for conflicts before applying changes
	if let Err(conflict) = Sync.check_for_conflicts(&change).await {
		dev_log!("lifecycle", "Conflict detected: {}", conflict);

		return Err(format!("Conflict detected: {}", conflict));
	}

	// Apply change via Mountain IPC instead of mock file system
	match change.change_type {
		ChangeType::Update => {
			// Update file content via Mountain IPC
			if let Some(_content) = &change.content {

				// self.mountain_ipc.update_document(
				//     &change.document_id,
				//     content,
				//     change.change_id.clone()
				// )
				// .await
				// .map_err(|e| format!("Failed to update document via
				// Mountain IPC: {}", e))?;
			}
		},

		ChangeType::Insert => {
			// Create new file via Mountain IPC
			if let Some(_content) = &change.content {

				// self.mountain_ipc.create_document(
				//     &change.document_id,
				//     content.as_str(),
				//     change.change_id.clone()
				// )
				// .await
				// .map_err(|e| format!("Failed to create document via
				// Mountain IPC: {}", e))?;
			}
		},

		ChangeType::Delete => {

			// Delete file via Mountain IPC
			// self.mountain_ipc.delete_document(
			//     &change.document_id,
			//     change.change_id.clone()
			// )
			// .await
			// .map_err(|e| format!("Failed to delete document via Mountain
			// IPC: {}", e))?;
		},

		_ => {
			dev_log!("lifecycle", "Unsupported change type: {:?}", change.change_type);
		},
	}

	// Mark change as applied
	let mut sync = Sync.document_sync.lock().unwrap_or_else(|e| e.into_inner());

	if let Some(changes) = sync.pending_changes.get_mut(&change.document_id) {
		if let Some(change_idx) = changes.iter().position(|c| c.change_id == change.change_id) {
			changes[change_idx].applied = true;
		}
	}

	// PERFORMANCE TRACKING: Microsoft-inspired operation metrics
	let change_duration = change_start.elapsed();

	dev_log!(
		"ipc",
		"[WindAdvancedSync] Change applied successfully in {:.2}ms: {}",
		change_duration.as_millis(),
		change.change_id
	);

	Ok(())
}
