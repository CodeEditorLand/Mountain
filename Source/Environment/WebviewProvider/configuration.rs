//! # WebviewProvider - Configuration Operations
//!
//! Implementation of webview configuration methods for [`MountainEnvironment`](crate::MountainEnvironment::MountainEnvironment)
//!
//! Handles setting webview options and HTML content.

use std::collections::HashMap;

use CommonLibrary::{
	Error::CommonError::CommonError,
};
use log::debug;
use serde_json::{json, Value};
use tauri::{Emitter, Manager, WebviewWindow};

use super::super::{MountainEnvironment::MountainEnvironment, Utility};

/// Configuration operations implementation for MountainEnvironment
pub(super) async fn set_webview_options_impl(
	env: &MountainEnvironment,
	handle: String,
	options_value: Value,
) -> Result<(), CommonError> {
	debug!("[WebviewProvider] Setting options for Webview: {}", handle);

	if let Some(webview_window) = env.ApplicationHandle.get_webview_window(&handle) {
		let options_map: HashMap<String, Value> = serde_json::from_value(options_value.clone()).map_err(|error| {
			CommonError::SerializationError { Description: format!("Failed to parse Webview options: {}", error) }
		})?;

		// Update title
		if let Some(title) = options_map.get("title").and_then(|v| v.as_str()) {
			webview_window.set_title(title).map_err(|error| {
				CommonError::UserInterfaceInteraction { Reason: format!("Failed to set Webview title: {}", error) }
			})?;

			// Update state
			{
				let mut webview_guard = env
					.ApplicationState
					.Feature.Webviews.ActiveWebviews
					.lock()
					.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

				if let Some(state) = webview_guard.get_mut(&handle) {
					state.Title = title.to_string();
				}
			}
		}

		// Set the webview panel's icon by storing the icon path in the
		// webview state in ApplicationState.Feature.Webviews. Validate the
		// path exists, convert to appropriate format (Url or string),
		// update the UI to display the icon in the tab bar or title
		// area, and emit an event to refresh the frontend
		// representation. The icon path can be a theme-aware icon path or a
		// custom image file URI.
	}

	// Emit options changed event
	env.ApplicationHandle
		.emit::<Value>(
			"sky://webview/options-changed",
			json!({ "Handle": handle, "Options": options_value }),
		)
		.map_err(|error| {
			CommonError::IPCError { Description: format!("Failed to emit Webview options changed event: {}", error) }
		})?;

	Ok(())
}

/// Sets the HTML content of a Webview.
pub(super) async fn set_webview_html_impl(
	env: &MountainEnvironment,
	handle: String,
	html: String,
) -> Result<(), CommonError> {
	debug!("[WebviewProvider] Setting HTML for Webview: {} ({} bytes)", handle, html.len());

	if let Some(webview_window) = env.ApplicationHandle.get_webview_window(&handle) {
		webview_window.emit::<String>("sky://webview/set-html", html).map_err(|error| {
			CommonError::IPCError { Description: format!("Failed to set Webview HTML: {}", error) }
		})?;

		Ok(())
	} else {
		Err(CommonError::WebviewNotFound { Handle: handle })
	}
}
