use Common::{error::CommonError, webview::dto::*};
use log::{error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use uuid::Uuid;

/// @module WebviewLogic
/// @description Contains the core logic for creating and managing webview
/// instances, acting as a bridge between RPC calls from Cocoon and UI events
/// sent to Sky.
use crate::{
	AppState::{AppState::AppState, Dto::WebviewStateDto},
	handlers::error_utils,
	vine,
};

/// Logic to create a new webview panel. This is called by the `WebviewProvider`
/// in the environment.
pub async fn CreateWebviewPanelLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	ExtensionData:WebviewExtensionDescriptionDto,
	ViewType:String,
	Title:String,
	ShowOptions:WebviewShowOptionsDto,
	PanelOptions:WebviewPanelOptionsDto,
	ContentOptions:WebviewContentOptionsDto,
	_SerializeBuffersForPostMessage:bool,
) -> Result<String, CommonError> {
	let Handle = Uuid::new_v4().to_string();
	info!("[WebviewLogic] Creating webview panel with handle: {}", Handle);

	let NewState = WebviewStateDto {
		Handle:Handle.clone(),
		ViewType:ViewType.clone(),
		Title:Title.clone(),
		PanelOptions:PanelOptions.clone(),
		ContentOptions:ContentOptions.clone(),
		SidecarIdentifier:"cocoon-main".to_string(), // This could be dynamic in the future.
		ExtensionId:ExtensionData.Id.clone(),
		IsActive:true, // Panels are typically active on creation.
		IsVisible:true,
	};

	// Store the initial state of the webview.
	let AppStateInstance = AppHandle.state::<AppState>();
	AppStateInstance.ActiveWebviews.lock().unwrap().insert(Handle.clone(), NewState);

	// Emit an event to the Sky frontend, telling it to create the actual webview
	// UI.
	AppHandle
		.emit(
			"sky://webview/create",
			json!({
				"Handle": Handle,
				"ViewType": ViewType,
				"Title": Title,
				"ShowOptions": ShowOptions,
				"PanelOptions": PanelOptions,
				"ContentOptions": ContentOptions,
			}),
		)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;

	Ok(Handle)
}

/// Logic to set the HTML content of a webview.
pub async fn SetWebviewHtmlLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Handle:String,
	Html:String,
) -> Result<(), CommonError> {
	info!("[WebviewLogic] Setting HTML for webview: {}", Handle);
	AppHandle
		.emit("sky://webview/set-html", json!({ "Handle": Handle, "Html": Html }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

/// Logic to post a message from the extension host to the webview's content
/// script.
pub async fn PostMessageToWebviewLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Handle:String,
	Message:Value,
) -> Result<bool, CommonError> {
	info!("[WebviewLogic] Posting message to webview: {}", Handle);
	AppHandle
		.emit("sky://webview/post-message", json!({ "Handle": Handle, "Message": Message }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	Ok(true)
}

/// Logic to reveal an existing webview panel.
pub async fn RevealWebviewPanelLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Handle:String,
	ShowOptions:WebviewShowOptionsDto,
) -> Result<(), CommonError> {
	info!("[WebviewLogic] Revealing webview: {}", Handle);
	AppHandle
		.emit("sky://webview/reveal", json!({ "Handle": Handle, "ShowOptions": ShowOptions }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

/// Logic to dispose of a webview panel.
pub async fn DisposeWebviewLogic<R:Runtime>(AppHandle:&AppHandle<R>, Handle:String) -> Result<(), CommonError> {
	info!("[WebviewLogic] Disposing webview: {}", Handle);
	let AppStateInstance = AppHandle.state::<AppState>();
	AppStateInstance.ActiveWebviews.lock().unwrap().remove(&Handle);
	AppHandle
		.emit("sky://webview/dispose", json!({ "Handle": Handle }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
