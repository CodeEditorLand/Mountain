//! Cocoon `workspace.applyEdit` notification - extension called
//! `vscode.workspace.applyEdit(edit)` via the fire-and-forget notification
//! path (fallback when the primary `process_cocoon_request` path is
//! unavailable). Performs a round-trip to Sky so the edit is actually applied
//! to the Monaco model before the spawned task completes. The boolean result
//! is discarded here because the gRPC notification contract has no return
//! channel; callers that need the success boolean must use the request path
//! (`applyEdit` via `process_cocoon_request`).

use std::sync::Arc;

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WorkspaceApplyEdit(Service:&MountainVinegRPCService, Parameter:&Value) {
<<<<<<< HEAD
	let RunTime = Arc::clone(Service.RunTime());

	let Payload = Parameter.clone();

	// Spawn the round-trip on the async runtime so `send_cocoon_notification`
	// returns `Empty` immediately (gRPC notification contract), while the edit
	// is still applied asynchronously with a unique request-ID nonce that Sky
	// can use to resolve its `ResolveUiRequest` callback.
	tauri::async_runtime::spawn(async move {
		match crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
			&RunTime.Environment,
			"sky://workspace/applyEdit",
			Payload,
		)
		.await
		{
			Ok(_) => {
				dev_log!("ipc", "[WorkspaceApplyEdit] notification round-trip resolved");
			},

			Err(Error) => {
				dev_log!("ipc", "warn: [WorkspaceApplyEdit] notification round-trip failed: {:?}", Error);
			},
		}
	});
=======
	::Vine::Server::Notification::WorkspaceApplyEdit::WorkspaceApplyEdit(Service, Parameter).await;
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
}
