//! `WorkspaceDelta::DispatchDeltaWorkspaceFolders`

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::json;
use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	IPC::SkyEmit::Fn,
	Vine::Client,
	dev_log,
};

/// Dispatch `$deltaWorkspaceFolders` to Cocoon. Returns immediately if both
/// arrays are empty - no point waking the sidecar for a no-op mutation.
///
/// Errors are logged and swallowed: the workspace state is already updated by
/// the caller, so a failed notification should not roll the mutation back. The
/// log tag `[LandFix:WsDelta]` keeps the event grep-able in dev logs and is
/// deliberately consistent with `[LandFix:WsNs]` on the Cocoon side.
pub async fn Fn(Added:Vec<WorkspaceFolderStateDTO>, Removed:Vec<WorkspaceFolderStateDTO>) {
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
		Client::SendNotification::Fn("cocoon-main".to_string(), "$deltaWorkspaceFolders".to_string(), Payload).await
	{
		dev_log!(
			"workspaces",
			"warn: [LandFix:WsDelta] $deltaWorkspaceFolders notification failed: {}",
			Error
		);
	}
}
