// File: Mountain/Source/Environment/WebViewProvider.rs
// Role: Implements the `WebViewProvider` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Core logic for creating and managing WebView instances.
//   - Uses Tauri's multi-window capabilities to host WebView content.
//   - Manages WebView state in `ApplicationState` and pushes updates to the
//     frontend.

//! # WebViewProvider Implementation
//!
//! Implements the `WebViewProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for creating and managing WebView
//! instances using Tauri's multi-window capabilities.

#![allow(non_snake_case, non_camel_case_types)]

use Common::{Error::CommonError::CommonError, WebView::WebViewProvider::WebViewProvider};
use async_trait::async_trait;
use log::info;
use serde_json::{Value, json};
use tauri::{Emitter, Manager, WebviewWindowBuilder};

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::WebViewStateDTO::WebViewStateDTO;

#[async_trait]
impl WebViewProvider for MountainEnvironment {
	async fn CreateWebViewPanel(
		&self,

		ExtensionDataValue:Value,

		ViewType:String,

		Title:String,

		_ShowOptionsValue:Value,

		PanelOptionsValue:Value,

		ContentOptionsValue:Value,
	) -> Result<String, CommonError> {
		let Handle = uuid::Uuid::new_v4().to_string();

		info!("[WebViewProvider] Creating WebViewPanel with handle: {}", Handle);

		let State = WebViewStateDTO {
			Handle:Handle.clone(),

			ViewType,

			Title:Title.clone(),

			ContentOptions:serde_json::from_value(ContentOptionsValue)?,

			PanelOptions:PanelOptionsValue,

			// TODO: This should come from request context
			SideCarIdentifier:"cocoon-main".to_string(),

			ExtensionIdentifier:ExtensionDataValue
				.get("id")
				.and_then(|v| v.as_str())
				.unwrap_or_default()
				.to_string(),

			IsActive:true,

			IsVisible:true,
		};

		// Store the initial state.
		self.ApplicationState
			.ActiveWebViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(Handle.clone(), State.clone());

		// Create a new Tauri window for this webview.
		WebviewWindowBuilder::new(
			&self.ApplicationHandle,
			&Handle,
			tauri::WebviewUrl::App("WebviewHost.html".into()),
		)
		.title(Title)
		.initialization_script(&format!("window.__WEBVIEW_INITIAL_STATE__ = {}", json!(State)))
		.build()
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;

		Ok(Handle)
	}

	async fn DisposeWebViewPanel(&self, Handle:String) -> Result<(), CommonError> {
		info!("[WebViewProvider] Disposing WebViewPanel: {}", Handle);

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			WebviewWindow
				.close()
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
		}

		self.ApplicationState
			.ActiveWebViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&Handle);

		Ok(())
	}

	async fn RevealWebViewPanel(&self, Handle:String, _ShowOptionsValue:Value) -> Result<(), CommonError> {
		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			WebviewWindow
				.set_focus()
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
		}

		Ok(())
	}

	async fn SetWebViewOptions(&self, Handle:String, OptionsValue:Value) -> Result<(), CommonError> {
		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			if let Some(Title) = OptionsValue.get("title").and_then(|v| v.as_str()) {
				WebviewWindow
					.set_title(Title)
					.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
			}

			// TODO: Implement icon path setting.
		}

		Ok(())
	}

	async fn SetWebViewHTML(&self, Handle:String, HTML:String) -> Result<(), CommonError> {
		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			WebviewWindow
				.emit("sky://webview/set-html", HTML)
				.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })?;
		}

		Ok(())
	}

	async fn PostMessageToWebView(&self, Handle:String, Message:Value) -> Result<bool, CommonError> {
		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			WebviewWindow
				.emit("sky://webview/post-message", Message)
				.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })?;

			return Ok(true);
		}

		Ok(false)
	}
}
