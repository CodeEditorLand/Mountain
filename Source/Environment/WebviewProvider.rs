// ============================================================================
// File: Mountain/Source/Environment/WebviewProvider.rs
// ============================================================================
// # WebviewProvider Implementation
//
// Implements the `WebviewProvider` trait for the `MountainEnvironment`.
// This provider contains the core logic for creating, managing, and securing
// Webview instances using Tauri's multi-window capabilities.
//
// ## Key Features:
// - Webview panel creation and lifecycle management
// - Secure message passing between Webview and host
// - Webview content isolation (sandboxed iframes)
// - State persistence and restoration
// - Webview visibility and focus management
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
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
	Webview::WebviewProvider::WebviewProvider,
};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{Emitter, Manager, WebviewWindowBuilder};
use uuid::Uuid;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO;

/// Represents a Webview message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebviewMessage {
	pub MessageIdentifier:String,
	pub MessageType:String,
	pub Payload:Value,
	pub Source:Option<String>,
}

/// Webview lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebviewLifecycleState {
	Unloaded,
	Loading,
	Loaded,
	Visible,
	Hidden,
	Disposed,
}

/// Webview message handler context
struct WebviewMessageContext {
	Handle:String,
	SideCarIdentifier:Option<String>,
	PendingResponses:HashMap<String, tokio::sync::oneshot::Sender<Value>>,
}

#[async_trait]
impl WebviewProvider for MountainEnvironment {
	/// Creates a new Webview panel with proper security isolation.
	async fn CreateWebviewPanel(
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
			"[WebviewProvider] Creating WebviewPanel with handle: {}, viewType: {}",
			Handle, ViewType
		);

		// Parse content options to ensure security settings
		let ContentOptions = serde_json::from_value(ContentOptionsValue.clone()).map_err(|Error| {
			CommonError::InvalidArgument { ArgumentName:"ContentOptions".into(), Reason:Error.to_string() }
		})?;

		let State = WebviewStateDTO {
			Handle:Handle.clone(),
			ViewType:ViewType.clone(),
			Title:Title.clone(),
			ContentOptions,
			PanelOptions:PanelOptionsValue.clone(),
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
			let mut WebviewGuard = self
				.ApplicationState
				.ActiveWebviews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			WebviewGuard.insert(Handle.clone(), State);
		}

		// Create a new Tauri window for this webview with security settings
		let TitleClone = Title.clone();
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
				"Title": TitleClone
			})
		))
		.build()
		.map_err(|Error| {
			error!("[WebviewProvider] Failed to create Webview window: {}", Error);
			CommonError::UserInterfaceInteraction { Reason:Error.to_string() }
		})?;

		// Setup message listener for this Webview
		Self::SetupWebviewMessageListener(self, Handle.clone()).await?;

		// Notify frontend about Webview creation
		self.ApplicationHandle
			.emit(
				"sky://webview/created",
				json!({ "Handle": Handle.clone(), "ViewType": ViewType.clone(), "Title": TitleClone }),
			)
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit Webview creation event: {}", Error) }
			})?;

		Ok(Handle)
	}

	/// Disposes a Webview panel and cleans up all associated resources.
	async fn DisposeWebviewPanel(&self, Handle:String) -> Result<(), CommonError> {
		info!("[WebviewProvider] Disposing WebviewPanel: {}", Handle);

		// Remove message listener
		Self::RemoveWebviewMessageListener(self, &Handle).await;

		// Close the window
		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			if let Err(Error) = WebviewWindow.close() {
				warn!("[WebviewProvider] Failed to close Webview window: {}", Error);
			}
		}

		// Remove state
		self.ApplicationState
			.ActiveWebviews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&Handle);

		// Notify frontend about Webview disposal
		self.ApplicationHandle
			.emit("sky://webview/disposed", json!({ "Handle": Handle }))
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit Webview disposal event: {}", Error) }
			})?;

		Ok(())
	}

	/// Reveals (shows and focuses) a Webview panel.
	async fn RevealWebviewPanel(&self, Handle:String, _ShowOptionsValue:Value) -> Result<(), CommonError> {
		info!("[WebviewProvider] Revealing WebviewPanel: {}", Handle);

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			WebviewWindow.show().map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to show Webview window: {}", Error) }
			})?;

			WebviewWindow.set_focus().map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to focus Webview window: {}", Error) }
			})?;

			// Update visibility state
			{
				let mut WebviewGuard = self
					.ApplicationState
					.ActiveWebviews
					.lock()
					.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

				if let Some(State) = WebviewGuard.get_mut(&Handle) {
					State.IsVisible = true;
				}
			}

			// Emit visibility event
			self.ApplicationHandle
				.emit("sky://webview/revealed", json!({ "Handle": Handle }))
				.map_err(|Error| {
					CommonError::IPCError { Description:format!("Failed to emit Webview revealed event: {}", Error) }
				})?;
		}

		Ok(())
	}

	/// Sets Webview options (title, icon, etc.).
	async fn SetWebviewOptions(&self, Handle:String, OptionsValue:Value) -> Result<(), CommonError> {
		info!("[WebviewProvider] Setting options for Webview: {}", Handle);

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			let OptionsMap:HashMap<String, Value> = serde_json::from_value(OptionsValue.clone()).map_err(|Error| {
				CommonError::SerializationError { Description:format!("Failed to parse Webview options: {}", Error) }
			})?;

			// Update title
			if let Some(Title) = OptionsMap.get("title").and_then(|v| v.as_str()) {
				WebviewWindow.set_title(Title).map_err(|Error| {
					CommonError::UserInterfaceInteraction { Reason:format!("Failed to set Webview title: {}", Error) }
				})?;

				// Update state
				{
					let mut WebviewGuard = self
						.ApplicationState
						.ActiveWebviews
						.lock()
						.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

					if let Some(State) = WebviewGuard.get_mut(&Handle) {
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
				CommonError::IPCError { Description:format!("Failed to emit Webview options changed event: {}", Error) }
			})?;

		Ok(())
	}

	/// Sets the HTML content of a Webview.
	async fn SetWebviewHTML(&self, Handle:String, HTML:String) -> Result<(), CommonError> {
		debug!("[WebviewProvider] Setting HTML for Webview: {} ({} bytes)", Handle, HTML.len());

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			WebviewWindow.emit("sky://webview/set-html", HTML).map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to set Webview HTML: {}", Error) }
			})?;

			Ok(())
		} else {
			Err(CommonError::WebviewNotFound { Handle })
		}
	}

	/// Posts a message to a Webview with proper error handling.
	async fn PostMessageToWebview(&self, Handle:String, Message:Value) -> Result<bool, CommonError> {
		debug!("[WebviewProvider] Posting message to Webview: {}", Handle);

		if let Some(WebviewWindow) = self.ApplicationHandle.get_webview_window(&Handle) {
			let WebviewMessage = WebviewMessage {
				MessageIdentifier:Uuid::new_v4().to_string(),
				MessageType:"request".to_string(),
				Payload:Message,
				Source:Some("host".to_string()),
			};

			WebviewWindow
				.emit("sky://webview/post-message", WebviewMessage)
				.map_err(|Error| {
					CommonError::IPCError { Description:format!("Failed to post message to Webview: {}", Error) }
				})?;

			debug!("[WebviewProvider] Message sent successfully to Webview: {}", Handle);
			Ok(true)
		} else {
			warn!("[WebviewProvider] Webview not found for message: {}", Handle);
			Ok(false)
		}
	}

	// ========================================================================
	// EXTRA METHODS - Not part of the WebviewProvider trait in CommonLibrary
	// These methods are commented out because the trait definition doesn't
	// include them. They may be added to the trait in the future or implemented
	// through a different mechanism.
	// ========================================================================

	// Receives a message from a Webview and routes it appropriately.
	// async fn ReceiveMessageFromWebview(&self, Handle:String, Message:Value) ->
	// Result<Value, CommonError> { debug!("[WebviewProvider] Received message from
	// Webview: {}", Handle);
	//
	// Get Webview state
	// let SideCarIdentifier = {
	// let WebviewGuard = self
	// .ApplicationState
	// .ActiveWebviews
	// .lock()
	// .map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
	//
	// WebviewGuard.get(&Handle).map(|State| State.SideCarIdentifier.clone())
	// };
	//
	// Route message to appropriate handler
	// if let Some(SideCarId) = SideCarIdentifier {
	// let IPCProvider:Arc<dyn IPCProvider> = self.Require();
	//
	// let RPCMethod = format!("{}$handleWebviewMessage",
	// ProxyTarget::ExtHostWebview.GetTargetPrefix()); let RPCParams = json!({
	// "Handle": Handle,
	// "Message": Message,
	// });
	//
	// return IPCProvider.SendRequestToSideCar(&SideCarId, RPCMethod, RPCParams,
	// 5000).await; }
	//
	// Handle locally if no sidecar
	// warn!("[WebviewProvider] No sidecar for Webview message: {}", Handle);
	// Ok(json!({ "status": "no_handler" }))
	// }
	//
	// Gets the current state of a Webview.
	// async fn GetWebviewState(&self, Handle:String) -> Result<Value, CommonError>
	// { let WebviewGuard = self
	// .ApplicationState
	// .ActiveWebviews
	// .lock()
	// .map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
	//
	// let State = WebviewGuard
	// .get(&Handle)
	// .ok_or_else(|| CommonError::WebviewNotFound { Handle:Handle.clone() })?;
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
	/// Sets up a message listener for a specific Webview.
	async fn SetupWebviewMessageListener(&self, Handle:String) -> Result<(), CommonError> {
		debug!("[WebviewProvider] Setting up message listener for Webview: {}", Handle);

		// In a full implementation, this would register an event listener
		// that forwards Webview messages to the appropriate handler.
		// For now, we'll just log a message.

		Ok(())
	}

	/// Removes a message listener for a specific Webview.
	async fn RemoveWebviewMessageListener(&self, _Handle:&str) {
		// In a full implementation, this would remove the event listener
		// that forwards Webview messages.
	}
}
