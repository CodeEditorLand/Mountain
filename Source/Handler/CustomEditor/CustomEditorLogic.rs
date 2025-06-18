// @module CustomEditorLogic
// @description Contains the core logic for managing custom editor providers
// and the lifecycle of custom documents.

use Common::{error::CommonError, IPC::DTO::ProxyTarget};
use log::{info, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::{
	ApplicationState::{ApplicationState, DTO::CustomDocumentStateDTO},
	Environment::Utility,
	Vine::client,
};

// Logic to register a new custom editor provider.
pub async fn RegisterCustomEditorLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	view_type:String,
	_options:Common::custom_editor::DTO::CustomEditorOptionsDTO,
	_extension_id:String,
	_sidecar_id:String,
) -> Result<(), CommonError> {
	info!(
		"[CustomEditorLogic] Registering custom editor provider for view type: {}",
		view_type
	);
	// In a real implementation, we would store the provider's details in AppState.
	// For now, this is a no-op as the logic is primarily in Cocoon.
	Ok(())
}

// Logic to unregister a custom editor provider.
pub async fn UnregisterCustomEditorLogic<R:Runtime>(
	_app_handle:&AppHandle<R>,
	view_type:String,
) -> Result<(), CommonError> {
	info!(
		"[CustomEditorLogic] Unregistering custom editor provider for view type: {}",
		view_type
	);
	// In a real implementation, we would remove the provider's details from
	// AppState.
	Ok(())
}

// Logic to create a new custom document.
pub async fn CreateCustomDocumentLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	resource_uri_DTO:Value,
	view_type:String,
) -> Result<Value, CommonError> {
	info!("[CustomEditorLogic] Creating custom document for view type '{}'", view_type);
	let resource_uri = Utility::GetUrlFromUriDTO(&resource_uri_DTO)?;

	// Make an RPC call to Cocoon to resolve the custom document.
	// This lets the extension's provider load data, etc.
	let response = client::SendRequest(
		"cocoon-main".to_string(), // Assume one sidecar
		format!("{}$resolveCustomEditor", ProxyTarget::ExtHostCustomEditors.GetTargetPrefix()),
		json!([resource_uri_DTO, view_type]),
		30000, // 30-second timeout
	)
	.await?;

	// The response should be the initial state of the document.
	let document_state:CustomDocumentStateDTO = serde_json::from_value(response.clone()).map_err(|e| {
		CommonError::SerdeError {
			Description:format!("Failed to deserialize CustomDocumentStateDTO from Cocoon: {}", e),
		}
	})?;

	// Store this document in our active state.
	let app_state = app_handle.state::<ApplicationState>();
	app_state
		.ActiveCustomDocuments
		.lock()
		.unwrap()
		.insert(resource_uri.to_string(), document_state);

	// Notify the frontend to create the editor User Interface.
	app_handle.emit("sky://custom-editor/create", response).map_err(|e| {
		CommonError::UiInteraction { Reason:format!("Failed to emit create custom editor event: {}", e) }
	})?;

	Ok(Value::Null)
}
