//! Storage command dispatcher - handles storage:* commands.

use std::sync::Arc;

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Storage::{
	StorageDelete::Fn as StorageDelete,
	StorageGet::Fn as StorageGet,
	StorageGetItems::Fn as StorageGetItems,
	StorageKeys::Fn as StorageKeys,
	StorageSet::Fn as StorageSet,
	StorageUpdateItems::Fn as StorageUpdateItems,
};

/// Dispatches storage commands with tier routing support.
///
/// Handled commands:
/// - `storage:get` -> StorageGet
/// - `storage:set` -> StorageSet
/// - `storage:getItems` -> StorageGetItems
/// - `storage:updateItems` -> StorageUpdateItems
/// - `storage:delete` -> StorageDelete
/// - `storage:keys` -> StorageKeys
/// - `storage:optimize` → flush pending writes
/// - `storage:isUsed` (stub)
/// - `storage:close` (stub)
/// - `storage:onDidChangeItems` (stub)
/// - `storage:logStorage` (stub)
pub async fn dispatch_storage(
	runtime:Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

	command:&str,

	arguments:Vec<Value>,

	tier_routes_to_node:impl Fn(&str, &str) -> bool,
) -> Result<Value, String> {
	// Check tier routing for storage commands
	if tier_routes_to_node("TierStorage", "TierStorage") {
		// Forward to Cocoon - this is handled in main dispatcher
		return Err("TIER_ROUTE_TO_NODE".to_string());
	}

	match command {
		"storage:get" => StorageGet(runtime.clone(), arguments).await,

		"storage:set" => StorageSet(runtime.clone(), arguments).await,

		"storage:getItems" => StorageGetItems(runtime.clone(), arguments).await,

		"storage:updateItems" => StorageUpdateItems(runtime.clone(), arguments).await,

		"storage:optimize" => {
			// Flush pending debounced writes for both scopes immediately.
			// VS Code calls this before workspace close and hot-reload to
			// ensure state is fully persisted without waiting for the 100 ms
			// debounce window.
			crate::dev_log!("storage", "storage:optimize → flush");

			let GlobalPath = Some((*runtime.Environment.ApplicationState.GlobalMementoPath.lock()).clone());

			let WorkspacePath = (*runtime.Environment.ApplicationState.WorkspaceMementoPath.lock()).clone();

			let GlobalData = (*runtime.Environment.ApplicationState.Configuration.MementoGlobalStorage.lock()).clone();

			let WorkspaceData = (*runtime
				.Environment
				.ApplicationState
				.Configuration
				.MementoWorkspaceStorage
				.lock())
			.clone();

			crate::Environment::StorageProvider::FlushPendingWrites(
				GlobalPath,
				WorkspacePath,
				GlobalData,
				WorkspaceData,
			)
			.await;

			Ok(Value::Null)
		},

		"storage:isUsed" => {
			crate::dev_log!("storage", "storage:isUsed");

			Ok(Value::Null)
		},

		"storage:close" => {
			crate::dev_log!("storage", "storage:close");

			Ok(Value::Null)
		},

		"storage:delete" => StorageDelete(runtime.clone(), arguments).await,

		"storage:keys" => StorageKeys(runtime.clone()).await,

		"storage:onDidChangeItems" | "storage:logStorage" => {
			crate::dev_log!("storage-verbose", "{} (stub-ack)", command);

			Ok(Value::Null)
		},

		_ => Err(format!("Unknown storage command: {}", command)),
	}
}
