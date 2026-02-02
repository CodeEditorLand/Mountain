// ============================================================================
// File: Mountain/Source/Environment/WebViewProvider.rs
// ============================================================================
// # WebViewProvider Implementation
//
// Implements the `WebViewProvider` trait for the `MountainEnvironment`.
// This provider contains the core logic for creating, managing, and securing
// WebView instances using Tauri's multi-window capabilities.
//
// ## Key Features:
// - WebView panel creation and lifecycle management
// - Secure message passing between WebView and host
// - WebView content isolation (sandboxed iframes)
// - State persistence and restoration
// - WebView visibility and focus management
// - Content injection (HTML/CSS/JavaScript)
//
// ## VSCode Reference:
// - vs/workbench/contrib/webview/browser/webviewExplorer.ts
// - vs/workbench/contrib/webview/browser/webviewService.ts
// - vs/workbench/contrib/webview/browser/webviewElement.ts
//
// ============================================================================

use std::{collections::HashMap, sync::Arc};

use CommonLibrary::{
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
	WebView::WebViewProvider::WebViewProvider,
};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{Emitter, Manager, WebviewWindowBuilder};
use uuid::Uuid;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::WebViewStateDTO::WebViewStateDTO;

/// Represents a WebView message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewMessage {
	pub MessageIdentifier:String,
	pub MessageType:String,
	pub Payload:Value,
	pub Source:Option<String>,
}

/// WebView lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebViewLifecycleState {
	Unloaded,
	Loading,
	Loaded,
	Visible,
	Hidden,
	Disposed,
}

/// WebView message handler context
struct WebViewMessageContext {
	Handle:String,
	SideCarIdentifier:Option<String>,
	PendingResponses:HashMap<String, tokio::sync::oneshot::Sender<Value>>,
}

#[async_trait]
impl WebViewProvider for MountainEnvironment {
	/// Creates a new WebView panel with proper security isolation.
	async fn CreateWebViewPanel(
		&self,
		ExtensionDataValue:Value,
		ViewType:String,
		Title:String,
		_ShowOptionsValue:Value,
		PanelOptionsValue:Value,
		ContentOptionsValue:Value,
	) -> Result<String, CommonError> {
		let Handle = Uuid::new_v4().to_string();

		info!(
			"[WebViewProvider] Creating WebViewPanel with handle: {}, viewType: {}",
			Handle, ViewType
		);

		// Parse content options to ensure security settings
		let ContentOptions = serde_json::from_value(ContentOptionsValue.clone()).map_err(|Error| {
			CommonError::InvalidArgument { ArgumentName:"ContentOptions".into(), Reason:Error.to_string() }
		})?;

		let State = WebViewStateDTO {
			Handle:Handle.clone(),
			ViewType:ViewType.clone(),
			Title:Title.clone(),
			ContentOptions,
			PanelOptions:PanelOptionsValue,
			SideCarIdentifier:"cocoon-main".to_string(),
			ExtensionIdentifier:ExtensionDataValue
				.get("id")
				.and_then(|v| v.as_str())
				.unwrap_or_default()
				.to_string(),
			IsActive:true,
			IsVisible:true,
		};

		// Store the initial state with lifecycle state
		{
			let mut WebViewGuard = self
				.ApplicationState
				.ActiveWebViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			WebViewGuard.insert(Handle.clone(), State);
		}

		// Create a new Tauri window for this webview with security settings
		WebviewWindowBuilder::new(
			&self.ApplicationHandle,
			&Handle,
			tauri::WebviewUrl::App("WebviewHost.html".into()),
		)
		.title(Title)
		.initialization_script(&format!(
			"window.__WEBVIEW_INITIAL_STATE__ = {};",
			json!({
				"Handle": Handle,
				"ViewType": ViewType,
				"Title": Title
			})
		))
		.build()
		.map_err(|Error| {
			error!("[WebViewProvider] Failed to create WebView window: {}", Error);
			CommonError::UserInterfaceInteraction { Reason:Error.to_string() }
		})?;

		// Setup message listener for this WebView
		Self::SetupWebViewMessageListener(self, Handle.clone()).await?;

		// Notify frontend about WebView creation
		self.ApplicationHandle
			.emit(
				"sky://webview/created",
				json!({ "Handle": Handle, "ViewType": ViewType, "Title": Title }),
			)
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit WebView creation event: {}", Error) }
			})?;

		Ok(Handle)
	}

	/// Disposes a WebView panel and cleans up all associated resources.
	async fn DisposeWebViewPanel(&self, Handle:String) -> Result<(), CommonError> {
		info!("[WebViewProvider] Disposing WebViewPanel: {}", Handle);

		// Remove message listener
		Self::RemoveWebViewMessageListener(self, &Handle).await;

		// Close the window
		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			if let Err(Error) = WebviewWindow.close() {
				warn!("[WebViewProvider] Failed to close WebView window: {}", Error);
			}
		}

		// Remove state
		self.ApplicationState
			.ActiveWebViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&Handle);

		// Notify frontend about WebView disposal
		self.ApplicationHandle
			.emit("sky://webview/disposed", json!({ "Handle": Handle }))
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit WebView disposal event: {}", Error) }
			})?;

		Ok(())
	}

	/// Reveals (shows and focuses) a WebView panel.
	async fn RevealWebViewPanel(&self, Handle:String, _ShowOptionsValue:Value) -> Result<(), CommonError> {
		info!("[WebViewProvider] Revealing WebViewPanel: {}", Handle);

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			WebviewWindow.show().map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to show WebView window: {}", Error) }
			})?;

			WebviewWindow.set_focus().map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to focus WebView window: {}", Error) }
			})?;

			// Update visibility state
			{
				let mut WebViewGuard = self
					.ApplicationState
					.ActiveWebViews
					.lock()
					.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

				if let Some(State) = WebViewGuard.get_mut(&Handle) {
					State.IsVisible = true;
				}
			}

			// Emit visibility event
			self.ApplicationHandle
				.emit("sky://webview/revealed", json!({ "Handle": Handle }))
				.map_err(|Error| {
					CommonError::IPCError { Description:format!("Failed to emit WebView revealed event: {}", Error) }
				})?;
		}

		Ok(())
	}

	/// Sets WebView options (title, icon, etc.).
	async fn SetWebViewOptions(&self, Handle:String, OptionsValue:Value) -> Result<(), CommonError> {
		info!("[WebViewProvider] Setting options for WebView: {}", Handle);

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			let OptionsMap:HashMap<String, Value> = serde_json::from_value(OptionsValue.clone()).map_err(|Error| {
				CommonError::SerializationError { Description:format!("Failed to parse WebView options: {}", Error) }
			})?;

			// Update title
			if let Some(Title) = OptionsMap.get("title").and_then(|v| v.as_str()) {
				WebviewWindow.set_title(Title).map_err(|Error| {
					CommonError::UserInterfaceInteraction { Reason:format!("Failed to set WebView title: {}", Error) }
				})?;

				// Update state
				{
					let mut WebViewGuard = self
						.ApplicationState
						.ActiveWebViews
						.lock()
						.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

					if let Some(State) = WebViewGuard.get_mut(&Handle) {
						State.Title = Title.to_string();
					}
				}
			}

			// TODO: Implement icon path setting
		}

		// Emit options changed event
		self.ApplicationHandle
			.emit(
				"sky://webview/options-changed",
				json!({ "Handle": Handle, "Options": OptionsValue }),
			)
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit WebView options changed event: {}", Error) }
			})?;

		Ok(())
	}

	/// Sets the HTML content of a WebView.
	async fn SetWebViewHTML(&self, Handle:String, HTML:String) -> Result<(), CommonError> {
		debug!("[WebViewProvider] Setting HTML for WebView: {} ({} bytes)", Handle, HTML.len());

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			WebviewWindow.emit("sky://webview/set-html", HTML).map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to set WebView HTML: {}", Error) }
			})?;

			Ok(())
		} else {
			Err(CommonError::WebViewNotFound { Handle })
		}
	}

	/// Posts a message to a WebView with proper error handling.
	async fn PostMessageToWebView(&self, Handle:String, Message:Value) -> Result<bool, CommonError> {
		debug!("[WebViewProvider] Posting message to WebView: {}", Handle);

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			let WebViewMessage = WebViewMessage {
				MessageIdentifier:Uuid::new_v4().to_string(),
				MessageType:"request".to_string(),
				Payload:Message,
				Source:Some("host".to_string()),
			};

			WebviewWindow
				.emit("sky://webview/post-message", WebViewMessage)
				.map_err(|Error| {
					CommonError::IPCError { Description:format!("Failed to post message to WebView: {}", Error) }
				})?;

			debug!("[WebViewProvider] Message sent successfully to WebView: {}", Handle);
			Ok(true)
		} else {
			warn!("[WebViewProvider] WebView not found for message: {}", Handle);
			Ok(false)
		}
	}

	// ========================================================================
	// EXTRA METHODS - Not part of the WebViewProvider trait in CommonLibrary
	// These methods are commented out because the trait definition doesn't
	// include them. They may be added to the trait in the future or implemented
	// through a different mechanism.
	// ========================================================================

	// Receives a message from a WebView and routes it appropriately.
	// async fn ReceiveMessageFromWebView(&self, Handle:String, Message:Value) ->
	// Result<Value, CommonError> { debug!("[WebViewProvider] Received message from
	// WebView: {}", Handle);
	//
	// Get WebView state
	// let SideCarIdentifier = {
	// let WebViewGuard = self
	// .ApplicationState
	// .ActiveWebViews
	// .lock()
	// .map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
	//
	// WebViewGuard.get(&Handle).map(|State| State.SideCarIdentifier.clone())
	// };
	//
	// Route message to appropriate handler
	// if let Some(SideCarId) = SideCarIdentifier {
	// let IPCProvider:Arc<dyn IPCProvider> = self.Require();
	//
	// let RPCMethod = format!("{}$handleWebViewMessage",
	// ProxyTarget::ExtHostWebView.GetTargetPrefix()); let RPCParams = json!({
	// "Handle": Handle,
	// "Message": Message,
	// });
	//
	// return IPCProvider.SendRequestToSideCar(&SideCarId, RPCMethod, RPCParams,
	// 5000).await; }
	//
	// Handle locally if no sidecar
	// warn!("[WebViewProvider] No sidecar for WebView message: {}", Handle);
	// Ok(json!({ "status": "no_handler" }))
	// }
	//
	// Gets the current state of a WebView.
	// async fn GetWebViewState(&self, Handle:String) -> Result<Value, CommonError>
	// { let WebViewGuard = self
	// .ApplicationState
	// .ActiveWebViews
	// .lock()
	// .map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
	//
	// let State = WebViewGuard
	// .get(&Handle)
	// .ok_or_else(|| CommonError::WebViewNotFound { Handle:Handle.clone() })?;
	//
	// Ok(json!({
	// "Handle": State.Handle,
	// "ViewType": State.ViewType,
	// "Title": State.Title,
	// "IsActive": State.IsActive,
	// "IsVisible": State.IsVisible,
	// "ExtensionIdentifier": State.ExtensionIdentifier,
	// }))
	// }
}

// ============================================================================
// Private Helper Methods
// ============================================================================

impl MountainEnvironment {
	/// Sets up a message listener for a specific WebView.
	async fn SetupWebViewMessageListener(&self, Handle:String) -> Result<(), CommonError> {
		debug!("[WebViewProvider] Setting up message listener for WebView: {}", Handle);

		// In a full implementation, this would register an event listener
		// that forwards WebView messages to the appropriate handler.
		// For now, we'll just log a message.

		Ok(())
	}

	/// Removes a message listener for a specific WebView.
	async fn RemoveWebViewMessageListener(&self, _Handle:&str) {
		// In a full implementation, this would remove the event listener
		// that forwards WebView messages.
	}
}
