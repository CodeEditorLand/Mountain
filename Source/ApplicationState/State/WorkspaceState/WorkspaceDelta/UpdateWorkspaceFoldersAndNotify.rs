//! `WorkspaceDelta::UpdateWorkspaceFoldersAndNotify`

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::json;
use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	IPC::SkyEmit::Fn,
	Vine::Client,
	dev_log,
};

/// Convenience wrapper: update the state and fire the delta in one call.
///
/// Spawns the notification on the current tokio runtime so callers in sync
/// contexts (Tauri command handlers, boot-time seeding) don't have to build an
/// async scope just to reach Cocoon. If no runtime is available (very early
/// boot, unit tests), the notification is dropped - the state still mutates.
pub fn Fn(
	State:&crate::ApplicationState::Struct::WorkspaceState::WorkspaceState::Struct,

	Folders:Vec<WorkspaceFolderStateDTO>,
) {
	let (Added, Removed) = State.SetWorkspaceFoldersReturnDelta(Folders);

	if Added.is_empty() && Removed.is_empty() {
		return;
	}

	if let Ok(Handle) = tokio::runtime::Handle::try_current() {
		Handle.spawn(async move {
			DispatchDeltaWorkspaceFolders(Added, Removed).await;
		});
	} else {
		dev_log!(
			"workspaces",
			"warn: [LandFix:WsDelta] No tokio runtime available - delta dropped ({} added, {} removed)",
			Added.len(),
			Removed.len()
		);
	}
}
