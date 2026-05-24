//! `WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast`

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::json;
use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	IPC::SkyEmit::Fn,
	Vine::Client,
	dev_log,
};

/// Variant that additionally emits a `sky://workspaces/changed` Tauri event
/// so Wind/Sky can update their own caches (recent-folders list, sidebar
/// breadcrumb) without polling `workspaces:getFolders`. Preferred call site
/// whenever the caller already has an `AppHandle` in scope.
pub fn Fn<R:tauri::Runtime>(
	ApplicationHandle:&tauri::AppHandle<R>,

	State:&crate::ApplicationState::Struct::WorkspaceState::WorkspaceState::Struct,

	Folders:Vec<WorkspaceFolderStateDTO>,
) {
	// `tauri::Emitter` was previously imported here because the body
	// called `.emit(...)` directly. Now routed through `LogSkyEmit`
	// (which imports `Emitter` itself), so the local import would be
	// dead code - removed to keep the file warning-clean.
	let (Added, Removed) = State.SetWorkspaceFoldersReturnDelta(Folders);

	if Added.is_empty() && Removed.is_empty() {
		return;
	}

	let AddedWire:Vec<serde_json::Value> = Added.iter().map(FolderToWire).collect();

	let RemovedWire:Vec<serde_json::Value> = Removed.iter().map(FolderToWire).collect();

	let BroadcastPayload = serde_json::json!({
		"added": AddedWire.clone(),
		"removed": RemovedWire.clone(),
		"folders": State
			.GetWorkspaceFolders()
			.iter()
			.map(FolderToWire)
			.collect::<Vec<_>>(),
	});

	if let Err(Error) = LogSkyEmit(ApplicationHandle, SkyEvent::WorkspacesChanged.AsStr(), BroadcastPayload) {
		dev_log!(
			"workspaces",
			"warn: [LandFix:WsDelta] sky://workspaces/changed emit failed: {}",
			Error
		);
	}

	// Persist the additions into the recently-opened list so the next boot's
	// File → Open Recent menu and the Welcome screen can surface them.
	// Mirrors VS Code's `ElectronMainWorkspacesMainService` behaviour.
	PersistRecentlyOpened(&Added);

	if let Ok(Handle) = tokio::runtime::Handle::try_current() {
		Handle.spawn(async move {
			DispatchDeltaWorkspaceFolders(Added, Removed).await;
		});
	}
}
