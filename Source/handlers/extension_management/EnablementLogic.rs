use std::sync::Arc;

use Common::{
	environment::Requires,
	error::CommonError,
	ipc::dto::ProxyTarget,
	storage::{SetStorageItem, StorageProvider},
};
use log::{info, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};
use vs_platform_extensions_common_extensions::{EnablementState, ExtensionIdentifier};

/// @module EnablementLogic
/// @description Contains the core logic for managing the enablement state of
/// extensions, including persisting the state and notifying the extension host
/// of changes.
use crate::{
	AppState,
	environment::MountainEnvironment,
	handlers::error_utils,
	vine::{self, client},
};

const ENABLEMENT_STATE_STORAGE_KEY:&str = "extensions.enablement";

/// Logic to retrieve the enablement state for a given extension from persistent
/// storage.
pub async fn GetEnablementStateLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Environment:&MountainEnvironment,
	ExtensionIdentifier:ExtensionIdentifier,
) -> Result<Value, CommonError> {
	info!("[EnablementLogic] Getting enablement state for: {}", ExtensionIdentifier.Value);

	let Storage:Arc<dyn StorageProvider> = Environment.Require();
	let AllEnablementState = Storage.GetStorageValue(true, ENABLEMENT_STATE_STORAGE_KEY).await?;

	let StateMap = match AllEnablementState {
		Some(Value::Object(map)) => map,
		_ => return Ok(json!(EnablementState::EnabledGlobally as u32)), // Default to enabled
	};

	let CurrentState = StateMap
		.get(&ExtensionIdentifier.Value)
		.and_then(Value::as_u64)
		.map_or(EnablementState::EnabledGlobally, |s| EnablementState::from(s as u32));

	Ok(json!(CurrentState as u32))
}

/// Logic to set the enablement state for one or more extensions, persisting the
/// change and notifying the extension host.
pub async fn SetEnablementLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Environment:&MountainEnvironment,
	ExtensionsToUpdate:Vec<ExtensionIdentifier>,
	NewState:EnablementState,
) -> Result<Value, CommonError> {
	info!(
		"[EnablementLogic] Setting state for {} extensions to: {:?}",
		ExtensionsToUpdate.len(),
		NewState
	);
	let Storage:Arc<dyn StorageProvider> = Environment.Require();

	// --- Update persistent storage ---
	let AllEnablementState = Storage.GetStorageValue(true, ENABLEMENT_STATE_STORAGE_KEY).await?;
	let mut StateMap = match AllEnablementState {
		Some(Value::Object(map)) => map,
		_ => serde_json::Map::new(),
	};

	for Extension in &ExtensionsToUpdate {
		StateMap.insert(Extension.Value.clone(), json!(NewState as u32));
	}

	// Use the StorageProvider effect to save the updated map.
	Storage
		.UpdateStorageValue(true, ENABLEMENT_STATE_STORAGE_KEY.to_string(), Some(Value::Object(StateMap)))
		.await?;

	// --- Notify Cocoon ---
	let ChangedIds:Vec<String> = ExtensionsToUpdate.into_iter().map(|e| e.Value).collect();
	let NotificationMethod = format!(
		"{}.$acceptEnablementChanged",
		ProxyTarget::ExtHostExtensionEnablement.GetTargetPrefix()
	);
	if let Err(e) = client::SendNotification("cocoon-main".to_string(), NotificationMethod, json!(ChangedIds)).await {
		warn!("[EnablementLogic] Failed to notify Cocoon of enablement change: {}", e);
	}

	Ok(json!(true))
}
