//! # WorkspaceDelta
//!
//! Dispatches `$deltaWorkspaceFolders` notifications from Mountain to Cocoon
//! whenever the open workspace folder set mutates. Called by every site that
//! flips the folder list (boot-time seed, the `MountainWorkspaceOpen*`
//! commands, pick-folder navigation, Wind add/remove, and the Cocoon-driven
//! `$updateWorkspaceFolders` request).
//!
//! The delta is computed by
//! [`WorkspaceState::SetWorkspaceFoldersReturnDelta`] and shipped as a
//! fire-and-forget Vine notification: Cocoon's `NotificationHandler` converts
//! it into a `didChangeWorkspaceFolders` event on
//! `WorkspaceEventEmitter`, which powers every extension's
//! `vscode.workspace.onDidChangeWorkspaceFolders` subscription. The same
//! payload primes the local workspace snapshot in `WorkspaceNamespace` so
//! `vscode.workspace.workspaceFolders` returns the fresh list on subsequent
//! synchronous reads.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::json;

use crate::{ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO, IPC::SkyEmit::LogSkyEmit, Vine::Client, dev_log};

/// Serialisation shape matching the Cocoon-side Workspace shim. Mirrors the
/// camelCase DTO Sky already serialises for `workspaces:getFolders`, so the
/// Cocoon handler can pass the payload through to extension listeners without
/// renaming fields.
fn FolderToWire(Folder:&WorkspaceFolderStateDTO) -> serde_json::Value {
	json!({
		"uri": Folder.URI.to_string(),
		"name": Folder.GetDisplayName(),
		"index": Folder.Index,
	})
}

/// Dispatch `$deltaWorkspaceFolders` to Cocoon. Returns immediately if both
/// arrays are empty - no point waking the sidecar for a no-op mutation.
///
/// Errors are logged and swallowed: the workspace state is already updated by
/// the caller, so a failed notification should not roll the mutation back. The
/// log tag `[LandFix:WsDelta]` keeps the event grep-able in dev logs and is
/// deliberately consistent with `[LandFix:WsNs]` on the Cocoon side.
pub async fn DispatchDeltaWorkspaceFolders(Added:Vec<WorkspaceFolderStateDTO>, Removed:Vec<WorkspaceFolderStateDTO>) {
	if Added.is_empty() && Removed.is_empty() {
		return;
	}

	let AddedWire:Vec<serde_json::Value> = Added.iter().map(FolderToWire).collect();
	let RemovedWire:Vec<serde_json::Value> = Removed.iter().map(FolderToWire).collect();

	dev_log!(
		"workspaces",
		"[LandFix:WsDelta] $deltaWorkspaceFolders +{} -{} (first added={})",
		AddedWire.len(),
		RemovedWire.len(),
		Added.first().map(|F| F.URI.as_str()).unwrap_or("<none>")
	);

	let Payload = json!({
		"added": AddedWire,
		"removed": RemovedWire,
	});

	if let Err(Error) =
		Client::SendNotification("cocoon-main".to_string(), "$deltaWorkspaceFolders".to_string(), Payload).await
	{
		dev_log!(
			"workspaces",
			"warn: [LandFix:WsDelta] $deltaWorkspaceFolders notification failed: {}",
			Error
		);
	}
}

/// Convenience wrapper: update the state and fire the delta in one call.
///
/// Spawns the notification on the current tokio runtime so callers in sync
/// contexts (Tauri command handlers, boot-time seeding) don't have to build an
/// async scope just to reach Cocoon. If no runtime is available (very early
/// boot, unit tests), the notification is dropped - the state still mutates.
pub fn UpdateWorkspaceFoldersAndNotify(
	State:&crate::ApplicationState::State::WorkspaceState::WorkspaceState::State,
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

/// Variant that additionally emits a `sky://workspaces/changed` Tauri event
/// so Wind/Sky can update their own caches (recent-folders list, sidebar
/// breadcrumb) without polling `workspaces:getFolders`. Preferred call site
/// whenever the caller already has an `AppHandle` in scope.
pub fn UpdateWorkspaceFoldersAndBroadcast<R:tauri::Runtime>(
	ApplicationHandle:&tauri::AppHandle<R>,
	State:&crate::ApplicationState::State::WorkspaceState::WorkspaceState::State,
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
	if let Err(Error) = LogSkyEmit(
		ApplicationHandle,
		SkyEvent::WorkspacesChanged.AsStr(),
		BroadcastPayload,
	) {
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

/// Append every folder in `Added` to `~/.land/workspaces/RecentlyOpened.json`,
/// deduping by URI and capping at 50 entries (the VS Code default). Swallows
/// every error - a failed write must not prevent the workspace change.
fn PersistRecentlyOpened(Added:&[WorkspaceFolderStateDTO]) {
	if Added.is_empty() {
		return;
	}
	let Home = std::env::var("HOME")
		.or_else(|_| std::env::var("USERPROFILE"))
		.unwrap_or_default();
	if Home.is_empty() {
		return;
	}
	let Path = std::path::PathBuf::from(Home)
		.join(".land")
		.join("workspaces")
		.join("RecentlyOpened.json");
	let mut Current:serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&Path)
		.ok()
		.and_then(|Contents| serde_json::from_str::<serde_json::Value>(&Contents).ok())
		.and_then(|V| V.as_object().cloned())
		.unwrap_or_default();
	let mut Workspaces = Current
		.get("workspaces")
		.and_then(|V| V.as_array())
		.cloned()
		.unwrap_or_default();
	for Folder in Added {
		let Uri = Folder.URI.to_string();
		Workspaces.retain(|Entry| Entry.get("uri").and_then(|V| V.as_str()).unwrap_or("") != Uri);
		Workspaces.insert(
			0,
			serde_json::json!({
				"uri": Uri,
				"label": Folder.GetDisplayName(),
			}),
		);
	}
	Workspaces.truncate(50);
	Current.insert("workspaces".into(), serde_json::Value::Array(Workspaces));
	if !Current.contains_key("files") {
		Current.insert("files".into(), serde_json::json!([]));
	}
	if let Some(Parent) = Path.parent() {
		let _ = std::fs::create_dir_all(Parent);
	}
	if let Ok(Serialised) = serde_json::to_vec_pretty(&serde_json::Value::Object(Current)) {
		let _ = std::fs::write(&Path, Serialised);
	}
}
