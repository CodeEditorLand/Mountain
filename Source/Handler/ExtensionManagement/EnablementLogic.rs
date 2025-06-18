// @module EnablementLogic
// @description Contains the core logic for managing the enablement state of
// extensions, including persisting the state and notifying the extension host
// of changes.

use std::sync::Arc;

use Common::{Environment::Requires, error::CommonError, IPC::DTO::ProxyTarget, storage::StorageProvider};
use log::{info, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};
use vs_platform_extensions_Common_extensions::{EnablementState, ExtensionIdentifier};

use crate::{Environment::MountainEnvironment, Vine::client};

const ENABLEMENT_STATE_STORAGE_KEY:&str = "extensions.enablement";

/// Logic to retrieve the enablement state for a given extension from persistent
/// storage.
pub async fn GetEnablementStateLogic<R:Runtime>(
	_app_handle:&AppHandle<R>,
	Environment:&MountainEnvironment,
	extension_identifier:ExtensionIdentifier,
) -> Result<Value, CommonError> {
	info!("[EnablementLogic] Getting enablement state for: {}", extension_identifier.Value);

	let storage:Arc<dyn StorageProvider> = Environment.Require();
	let all_enablement_state = storage.GetStorageValue(true, ENABLEMENT_STATE_STORAGE_KEY).await?;

	let state_map = match all_enablement_state {
		Some(Value::Object(map)) => map,
		_ => return Ok(json!(EnablementState::EnabledGlobally as u32)), // Default to enabled
	};

	let current_state = state_map
		.get(&extension_identifier.Value)
		.and_then(Value::as_u64)
		.map_or(EnablementState::EnabledGlobally, |s| EnablementState::from(s as u32));

	Ok(json!(current_state as u32))
}

/// Logic to set the enablement state for one or more extensions, persisting the
/// change and notifying the extension host.
pub async fn SetEnablementLogic<R:Runtime>(
	_app_handle:&AppHandle<R>,
	Environment:&MountainEnvironment,
	extensions_to_update:Vec<ExtensionIdentifier>,
	new_state:EnablementState,
) -> Result<Value, CommonError> {
	info!(
		"[EnablementLogic] Setting state for {} extensions to: {:?}",
		extensions_to_update.len(),
		new_state
	);
	let storage:Arc<dyn StorageProvider> = Environment.Require();

	// --- Update persistent storage ---
	let all_enablement_state = storage.GetStorageValue(true, ENABLEMENT_STATE_STORAGE_KEY).await?;
	let mut state_map = match all_enablement_state {
		Some(Value::Object(map)) => map,
		_ => serde_json::Map::new(),
	};

	for extension in &extensions_to_update {
		state_map.insert(extension.Value.clone(), json!(new_state as u32));
	}

	// Use the StorageProvider effect to save the updated map.
	storage
		.UpdateStorageValue(true, ENABLEMENT_STATE_STORAGE_KEY.to_string(), Some(Value::Object(state_map)))
		.await?;

	// --- Notify Cocoon ---
	let changed_ids:Vec<String> = extensions_to_update.into_iter().map(|e| e.Value).collect();
	let notification_method = format!(
		"{}.$acceptEnablementChanged",
		ProxyTarget::ExtHostExtensionEnablement.GetTargetPrefix()
	);
	if let Err(e) = client::SendNotification("cocoon-main".to_string(), notification_method, json!(changed_ids)).await {
		warn!("[EnablementLogic] Failed to notify Cocoon of enablement change: {}", e);
	}

	Ok(json!(true))
}
