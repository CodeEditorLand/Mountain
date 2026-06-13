//! Storage command router.
//!
//! Routes the storage IPC commands to their handler functions.
//! Returns `None` for unrecognised commands so the dispatcher can
//! fall through to stub handlers or the generic prefix guard.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{
	Environment::StorageProvider::FlushPendingWrites,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};
use super::{
	StorageDelete::Fn as StorageDelete,
	StorageGet::Fn as StorageGet,
	StorageGetItems::Fn as StorageGetItems,
	StorageKeys::Fn as StorageKeys,
	StorageSet::Fn as StorageSet,
	StorageUpdateItems::Fn as StorageUpdateItems,
};

/// Routes storage commands.  Returns `Some(result)` for handled
/// commands, `None` otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	ApplicationHandle:tauri::AppHandle,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"storage:get" => Some(StorageGet(RunTime, Arguments).await),

		"storage:set" => Some(StorageSet(RunTime, Arguments).await),

		"storage:getItems" => Some(StorageGetItems(RunTime, Arguments).await),

		"storage:updateItems" => Some(StorageUpdateItems(RunTime, Arguments).await),

		"storage:delete" => Some(StorageDelete(RunTime, Arguments).await),

		"storage:keys" => Some(StorageKeys(RunTime).await),

		"storage:optimize" => {
			// Flush pending debounced writes for both scopes immediately.
			// VS Code calls this before workspace close and hot-reload to
			// ensure state is fully persisted without waiting for the 100 ms
			// debounce window.
			dev_log!("storage", "storage:optimize → flush");

			let GlobalPath = Some((*RunTime.Environment.ApplicationState.GlobalMementoPath.lock()).clone());

			let WorkspacePath = (*RunTime.Environment.ApplicationState.WorkspaceMementoPath.lock()).clone();

			let GlobalData =
				(*RunTime.Environment.ApplicationState.Configuration.MementoGlobalStorage.lock()).clone();

			let WorkspaceData = (*RunTime
				.Environment
				.ApplicationState
				.Configuration
				.MementoWorkspaceStorage
				.lock())
			.clone();

			FlushPendingWrites(
				GlobalPath,
				WorkspacePath,
				GlobalData,
				WorkspaceData,
			)
			.await;

			Some(Ok(Value::Null))
		},

		"storage:isUsed" => {
			dev_log!("storage", "storage:isUsed");

			Some(Ok(Value::Null))
		},

		"storage:close" => {
			dev_log!("storage", "storage:close");

			Some(Ok(Value::Null))
		},

		// Stock VS Code exposes `onDidChangeItems` as a channel
		// event. Ack the listen-request; real change delivery is
		// via Tauri event elsewhere.
		"storage:onDidChangeItems" | "storage:logStorage" => {
			dev_log!("storage-verbose", "{} (stub-ack)", command);

			// Emit `sky://storage/changed` so Wind's listen() bridge
			// receives storage events for reactive consumers.
			let Payload = Arguments.first().cloned().unwrap_or(Value::Null);

			let _ = ApplicationHandle.emit("sky://storage/changed", &Payload);

			Some(Ok(Value::Null))
		},

		_ => None,
	}
}
