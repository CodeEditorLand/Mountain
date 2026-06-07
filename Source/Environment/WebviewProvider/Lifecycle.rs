//! # WebviewProvider - Lifecycle Operations
//!
//! Implementation of webview panel lifecycle methods for
//! [`MountainEnvironment`]
//!
//! Handles creation, disposal, and visibility management of webview panels.

use CommonLibrary::{
	Error::CommonError::CommonError,
	IPC::SkyEvent::SkyEvent,
	Webview::DTO::WebviewContentOptionsDTO::WebviewContentOptionsDTO,
};

use serde_json::{Value, json};

use tauri::{Emitter, Manager, WebviewWindowBuilder};

use uuid::Uuid;

use super::super::{MountainEnvironment::MountainEnvironment, Utility};

use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

/// Lifecycle operations implementation for MountainEnvironment
pub(super) async fn create_webview_panel_impl(
	env:&MountainEnvironment,

	extension_data_value:Value,

	view_type:String,

	title:String,

	_show_options_value:Value,

	panel_options_value:Value,

	content_options_value:Value,
) -> Result<String, CommonError> {

	let handle = Uuid::new_v4().to_string();

	dev_log!(
		"extensions",

		"[WebviewProvider] Creating WebviewPanel with handle: {}, viewType: {}",

		handle,

		view_type
	);

	// Parse content options to ensure security settings
	let content_options:WebviewContentOptionsDTO =
		serde_json::from_value(content_options_value.clone()).map_err(|error| {
			CommonError::InvalidArgument { ArgumentName:"ContentOptions".into(), Reason:error.to_string() }
		})?;

	let state = WebviewStateDTO {
		Handle:handle.clone(),

		ViewType:view_type.clone(),

		Title:title.clone(),

		ContentOptions:content_options,

		PanelOptions:panel_options_value.clone(),

		SideCarIdentifier:"cocoon-main".to_string(),

		ExtensionIdentifier:extension_data_value
			.get("id")
			.and_then(|v| v.as_str())
			.unwrap_or_default()
			.to_string(),

		IsActive:true,

		IsVisible:true,
	};

	// Store the initial state with lifecycle state
	{
		let mut webview_guard = env
			.ApplicationState
			.Feature
			.Webviews
			.ActiveWebviews
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		webview_guard.insert(handle.clone(), state);
	}

	// Create a new Tauri window for this webview with security settings.
	// Use the localhost plugin URL (http://localhost:PORT/Mountain/WebviewHost.html)
	// so the webview's scripts are served by the same localhost plugin as the
	// main window. WebviewUrl::App("WebviewHost.html") routes through
	// tauri::manager which has no assets when frontendDist is null.
	let title_clone = title.clone();

	let WebviewHostUrl = crate::IPC::WindServiceHandlers::Utilities::LocalhostUrl::Get::Fn()
		.map(|Base| format!("{}/Mountain/WebviewHost", Base))
		.unwrap_or_else(|| "http://localhost:15536/Mountain/WebviewHost".to_string());

	let WebviewUrlParsed = WebviewHostUrl
		.parse::<url::Url>()
		.map(tauri::WebviewUrl::External)
		.unwrap_or_else(|_| tauri::WebviewUrl::App("WebviewHost.html".into()));

	let _webview_window = WebviewWindowBuilder::new(&env.ApplicationHandle, &handle, WebviewUrlParsed)
		.title(title)
		.initialization_script(&format!(
			"window.__WEBVIEW_INITIAL_STATE__ = {};",

			json!({
				"Handle": handle,
				"ViewType": view_type,
				"Title": title_clone
			})
		))
		.build()
		.map_err(|error| {
			dev_log!(
				"extensions",

				"error: [WebviewProvider] Failed to create Webview window: {}",

				error
			);

			CommonError::UserInterfaceInteraction { Reason:error.to_string() }
		})?;

	// Setup message listener for this Webview
	crate::Environment::WebviewProvider::Messaging::setup_webview_message_listener_impl(env, handle.clone()).await?;

	// Notify frontend about Webview creation
	env.ApplicationHandle
		.emit::<Value>(
			SkyEvent::WebviewCreated.AsStr(),

			json!({ "Handle": handle.clone(), "ViewType": view_type.clone(), "Title": title_clone }),
		)
		.map_err(|error| {
			CommonError::IPCError { Description:format!("Failed to emit Webview creation event: {}", error) }
		})?;

	Ok(handle)
}

/// Disposes a Webview panel and cleans up all associated resources.
pub(super) async fn dispose_webview_panel_impl(env:&MountainEnvironment, handle:String) -> Result<(), CommonError> {

	dev_log!("extensions", "[WebviewProvider] Disposing WebviewPanel: {}", handle);

	// Remove message listener
	let _ = crate::Environment::WebviewProvider::Messaging::remove_webview_message_listener_impl(env, &handle).await;

	// Close the window
	if let Some(webview_window) = env.ApplicationHandle.get_webview_window(&handle) {
		if let Err(error) = webview_window.close() {
			dev_log!(
				"extensions",

				"warn: [WebviewProvider] Failed to close Webview window: {}",

				error
			);
		}
	}

	// Remove state
	env.ApplicationState
		.Feature
		.Webviews
		.ActiveWebviews
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
		.remove(&handle);

	// Notify frontend about Webview disposal
	env.ApplicationHandle
		.emit::<Value>(SkyEvent::WebviewDisposed.AsStr(), json!({ "Handle": handle }))
		.map_err(|error| {
			CommonError::IPCError { Description:format!("Failed to emit Webview disposal event: {}", error) }
		})?;

	Ok(())
}

/// Reveals (shows and focuses) a Webview panel.
pub(super) async fn reveal_webview_panel_impl(
	env:&MountainEnvironment,

	handle:String,

	_show_options_value:Value,
) -> Result<(), CommonError> {

	dev_log!("extensions", "[WebviewProvider] Revealing WebviewPanel: {}", handle);

	if let Some(webview_window) = env.ApplicationHandle.get_webview_window(&handle) {
		webview_window.show().map_err(|error| {
			CommonError::UserInterfaceInteraction { Reason:format!("Failed to show Webview window: {}", error) }
		})?;

		webview_window.set_focus().map_err(|error| {
			CommonError::UserInterfaceInteraction { Reason:format!("Failed to focus Webview window: {}", error) }
		})?;

		// Update visibility state
		{
			let mut webview_guard = env
				.ApplicationState
				.Feature
				.Webviews
				.ActiveWebviews
				.lock()
				.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

			if let Some(state) = webview_guard.get_mut(&handle) {
				state.IsVisible = true;
			}
		}

		// Emit visibility event
		env.ApplicationHandle
			.emit::<Value>(SkyEvent::WebviewRevealed.AsStr(), json!({ "Handle": handle }))
			.map_err(|error| {
				CommonError::IPCError { Description:format!("Failed to emit Webview revealed event: {}", error) }
			})?;
	}

	Ok(())
}
