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

use Common::{Error::CommonError::CommonError, WebView::WebViewProvider::WebViewProvider};
use async_trait::async_trait;
use log::{error, info};
use serde_json::{Value, json};
use tauri::{Emitter, Manager, WebviewWindowBuilder};

use super::MountainEnvironment::MountainEnvironment;
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
			.unwrap()
			.insert(Handle.clone(), State.clone());

		// Create a new Tauri window for this webview.
		let WindowResult = WebviewWindowBuilder::new(
			&self.ApplicationHandle,
			&Handle,
			tauri::WebviewUrl::App("WebviewHost.html".into()),
		)
		.title(Title)
		.initialization_script(&format!("window.__WEBVIEW_INITIAL_STATE__ = {}", json!(State)))
		.build();

		if let Err(e) = WindowResult {
			error!("[WebViewProvider] Failed to create webview window: {}", e);

			return Err(CommonError::UserInterfaceInteraction { Reason:e.to_string() });
		}

		Ok(Handle)
	}

	async fn DisposeWebViewPanel(&self, Handle:String) -> Result<(), CommonError> {
		info!("[WebViewProvider] Disposing WebViewPanel: {}", Handle);

		if let Some(webview_window) = self.ApplicationHandle.get_webview_window(&Handle) {
			webview_window
				.close()
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		}

		self.ApplicationState.ActiveWebViews.lock().unwrap().remove(&Handle);

		Ok(())
	}

	async fn RevealWebViewPanel(&self, Handle:String, _ShowOptionsValue:Value) -> Result<(), CommonError> {
		if let Some(webview_window) = self.ApplicationHandle.get_webview_window(&Handle) {
			webview_window
				.set_focus()
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		}

		Ok(())
	}

	async fn SetWebViewOptions(&self, Handle:String, OptionsValue:Value) -> Result<(), CommonError> {
		if let Some(webview_window) = self.ApplicationHandle.get_webview_window(&Handle) {
			if let Some(title) = OptionsValue.get("title").and_then(|v| v.as_str()) {
				webview_window.set_title(title).expect("");
			}

			// TODO: Implement icon path setting.
		}

		Ok(())
	}

	async fn SetWebViewHTML(&self, Handle:String, HTML:String) -> Result<(), CommonError> {
		if let Some(webview_window) = self.ApplicationHandle.get_webview_window(&Handle) {
			webview_window
				.emit("sky://webview/set-html", HTML)
				.map_err(|e| CommonError::IPCError { Description:e.to_string() })?;
		}

		Ok(())
	}

	async fn PostMessageToWebView(&self, Handle:String, Message:Value) -> Result<bool, CommonError> {
		if let Some(webview_window) = self.ApplicationHandle.get_webview_window(&Handle) {
			webview_window
				.emit("sky://webview/post-message", Message)
				.map_err(|e| CommonError::IPCError { Description:e.to_string() })?;

			return Ok(true);
		}

		Ok(false)
	}
}
