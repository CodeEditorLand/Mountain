use Common::{error::CommonError, webview::dto::*};
use log::{error, info};
use serde_json::{Value, json};
use tauri::{ApplicationHandle, Emitter, Manager, RunTime};
use uuid::Uuid;

// @module WebviewLogic
// @description Contains the core logic for creating and managing webview
// instances, acting as a bridge between RPC calls from Cocoon and UI events
// sent to Sky.
use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::WebviewStateDto},
	Handler::error_utils,
	vine,
};

// Logic to create a new webview panel. This is called by the `WebviewProvider`
// in the environment.
pub async fn CreateWebviewPanelLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
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
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	AppStateInstance.ActiveWebviews.lock().unwrap().insert(Handle.clone(), NewState);

	// Emit an event to the Sky frontend, telling it to create the actual webview
	// UI.
	ApplicationHandle
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

// Logic to set the HTML content of a webview.
pub async fn SetWebviewHtmlLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Handle:String,
	Html:String,
) -> Result<(), CommonError> {
	info!("[WebviewLogic] Setting HTML for webview: {}", Handle);
	ApplicationHandle
		.emit("sky://webview/set-html", json!({ "Handle": Handle, "Html": Html }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to post a message from the extension host to the webview's content
// script.
pub async fn PostMessageToWebviewLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Handle:String,
	Message:Value,
) -> Result<bool, CommonError> {
	info!("[WebviewLogic] Posting message to webview: {}", Handle);
	ApplicationHandle
		.emit("sky://webview/post-message", json!({ "Handle": Handle, "Message": Message }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	Ok(true)
}

// Logic to reveal an existing webview panel.
pub async fn RevealWebviewPanelLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Handle:String,
	ShowOptions:WebviewShowOptionsDto,
) -> Result<(), CommonError> {
	info!("[WebviewLogic] Revealing webview: {}", Handle);
	ApplicationHandle
		.emit("sky://webview/reveal", json!({ "Handle": Handle, "ShowOptions": ShowOptions }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to dispose of a webview panel.
pub async fn DisposeWebviewLogic<R:RunTime>(ApplicationHandle:&ApplicationHandle<R>, Handle:String) -> Result<(), CommonError> {
	info!("[WebviewLogic] Disposing webview: {}", Handle);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	AppStateInstance.ActiveWebviews.lock().unwrap().remove(&Handle);
	ApplicationHandle
		.emit("sky://webview/dispose", json!({ "Handle": Handle }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
