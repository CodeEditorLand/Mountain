// @module WebViewLogic
// @description Contains the core logic for creating and managing webview
// instances, acting as a bridge between RPC calls from Cocoon and User Interface events
// sent to Sky.

use Common::{error::CommonError, webview::DTO::*};
use log::{error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use uuid::Uuid;

use crate::ApplicationState::{ApplicationState::ApplicationState, DTO::WebViewStateDTO};

// Logic to create a new webview panel. This is called by the `WebViewProvider`
// in the Environment.
pub async fn CreateWebViewPanelLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	extension_data:WebViewExtensionDescriptionDTO,
	view_type:String,
	title:String,
	show_options:WebViewShowOptionsDTO,
	panel_options:WebViewPanelOptionsDTO,
	content_options:WebViewContentOptionsDTO,
	_serialize_buffers_for_post_message:bool,
) -> Result<String, CommonError> {
	let handle = Uuid::new_v4().to_string();
	info!("[WebViewLogic] Creating webview panel with handle: {}", handle);

	let new_state = WebViewStateDTO {
		Handle:handle.clone(),
		ViewType:view_type.clone(),
		Title:title.clone(),
		PanelOptions:panel_options.clone(),
		ContentOptions:content_options.clone(),
		SidecarIdentifier:"cocoon-main".to_string(), // This could be dynamic in the future.
		ExtensionId:extension_data.Id.clone(),
		IsActive:true, // Panels are typically active on creation.
		IsVisible:true,
	};

	// Store the initial state of the webview.
	let app_state = app_handle.state::<ApplicationState>();
	app_state.ActiveWebViews.lock().unwrap().insert(handle.clone(), new_state);

	// Emit an event to the Sky frontend, telling it to create the actual webview
	// User Interface.
	app_handle
		.emit(
			"sky://webview/create",
			json!({
				"Handle": handle,
				"ViewType": view_type,
				"Title": title,
				"ShowOptions": show_options,
				"PanelOptions": panel_options,
				"ContentOptions": content_options,
			}),
		)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;

	Ok(handle)
}

// Logic to set the HTML content of a webview.
pub async fn SetWebViewHtmlLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	handle:String,
	html:String,
) -> Result<(), CommonError> {
	info!("[WebViewLogic] Setting HTML for webview: {}", handle);
	app_handle
		.emit("sky://webview/set-html", json!({ "Handle": handle, "Html": html }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to post a message from the extension host to the webview's content
// script.
pub async fn PostMessageToWebViewLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	handle:String,
	message:Value,
) -> Result<bool, CommonError> {
	info!("[WebViewLogic] Posting message to webview: {}", handle);
	app_handle
		.emit("sky://webview/post-message", json!({ "Handle": handle, "Message": message }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	Ok(true)
}

// Logic to reveal an existing webview panel.
pub async fn RevealWebViewPanelLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	handle:String,
	show_options:WebViewShowOptionsDTO,
) -> Result<(), CommonError> {
	info!("[WebViewLogic] Revealing webview: {}", handle);
	app_handle
		.emit("sky://webview/reveal", json!({ "Handle": handle, "ShowOptions": show_options }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to set the title of a webview panel.
pub async fn SetWebViewTitleLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	handle:String,
	title:String,
) -> Result<(), CommonError> {
	info!("[WebViewLogic] Setting title for webview {}: {}", handle, title);
	app_handle
		.emit("sky://webview/set-title", json!({ "Handle": handle, "Title": title }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to set the icon of a webview panel.
pub async fn SetWebViewIconPathLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	handle:String,
	icon_path:Option<Value>,
) -> Result<(), CommonError> {
	info!("[WebViewLogic] Setting icon for webview {}", handle);
	app_handle
		.emit("sky://webview/set-icon", json!({ "Handle": handle, "IconPath": icon_path }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to dispose of a webview panel.
pub async fn DisposeWebViewLogic<R:Runtime>(app_handle:&AppHandle<R>, handle:String) -> Result<(), CommonError> {
	info!("[WebViewLogic] Disposing webview: {}", handle);
	let app_state = app_handle.state::<ApplicationState>();
	app_state.ActiveWebViews.lock().unwrap().remove(&handle);
	app_handle
		.emit("sky://webview/dispose", json!({ "Handle": handle }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
