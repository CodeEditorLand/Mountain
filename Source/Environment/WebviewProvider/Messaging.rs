//! # WebviewProvider - Messaging Operations
//!
//! Implementation of webview message passing for
//! [`MountainEnvironment`]
//!
//! Handles secure bidirectional communication between host and webview.

use std::collections::HashMap;

use CommonLibrary::{Error::CommonError::CommonError, IPC::SkyEvent::SkyEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{Emitter, Listener, Manager};
use uuid::Uuid;

use super::super::MountainEnvironment::MountainEnvironment;
use crate::dev_log;

/// Represents a Webview message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebviewMessage {
	pub MessageIdentifier:String,

	pub MessageType:String,

	pub Payload:Value,

	pub Source:Option<String>,
}

/// Webview message handler context
#[allow(dead_code)]
struct WebviewMessageContext {
	Handle:String,

	SideCarIdentifier:Option<String>,

	PendingResponses:HashMap<String, tokio::sync::oneshot::Sender<Value>>,
}

/// Messaging operations implementation for MountainEnvironment
pub(super) async fn post_message_to_webview_impl(
	env:&MountainEnvironment,

	handle:String,

	message:Value,
) -> Result<bool, CommonError> {
	dev_log!("extensions", "[WebviewProvider] Posting message to Webview: {}", handle);

	if let Some(webview_window) = env.ApplicationHandle.get_webview_window(&handle) {
		let webview_message = WebviewMessage {
			MessageIdentifier:Uuid::new_v4().to_string(),

			MessageType:"request".to_string(),

			Payload:message,

			Source:Some("host".to_string()),
		};

		webview_window
			.emit::<WebviewMessage>(SkyEvent::WebviewPostMessage.AsStr(), webview_message)
			.map_err(|error| {
				CommonError::IPCError { Description:format!("Failed to post message to Webview: {}", error) }
			})?;

		dev_log!(
			"extensions",
			"[WebviewProvider] Message sent successfully to Webview: {}",
			handle
		);

		Ok(true)
	} else {
		dev_log!(
			"extensions",
			"warn: [WebviewProvider] Webview not found for message: {}",
			handle
		);

		Ok(false)
	}
}

/// Sets up a message listener for a specific Webview.
///
/// When an extension iframe calls `acquireVsCodeApi().postMessage(data)`,
/// the iframe's `pre/index.html` shim fires a `webview-message` Tauri event
/// on the webview window. We forward it to Cocoon via
/// `SendNotificationToSideCar("cocoon-main", "webview.message", {handle,
/// message})` so the extension host's `onDidReceiveMessage` subscriber fires.
pub(super) async fn setup_webview_message_listener_impl(
	env:&MountainEnvironment,

	handle:String,
) -> Result<(), CommonError> {
	dev_log!(
		"extensions",
		"[WebviewProvider] Setting up message listener for Webview: {}",
		handle
	);

	if let Some(WebviewWin) = env.ApplicationHandle.get_webview_window(&handle) {
		let H = handle.clone();

		WebviewWin.listen("webview-message", move |Event| {
			let H2 = H.clone();

			let RawPayload = Event.payload();

			let Parsed:Value = serde_json::from_str(RawPayload).unwrap_or_else(|_| {
				// If it's not valid JSON, wrap as a string value so Cocoon
				// still receives something meaningful.
				Value::String(RawPayload.to_string())
			});

			tokio::spawn(async move {
				let Notification = serde_json::json!({
					"handle": H2,
					"message": Parsed,
				});

				if let Err(E) = crate::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"webview.message".to_string(),
					Notification,
				)
				.await
				{
					dev_log!(
						"extensions",
						"warn: [WebviewProvider] webview.message notify failed handle={}: {}",
						H2,
						E
					);
				}
			});
		});

		dev_log!(
			"extensions",
			"[WebviewProvider] Message listener installed for handle={}",
			handle
		);
	} else {
		dev_log!(
			"extensions",
			"warn: [WebviewProvider] Webview window not found for handle={}, listener skipped",
			handle
		);
	}

	Ok(())
}

/// Removes a message listener for a specific Webview.
/// Tauri's `listen` returns an unlisten closure; for simplicity we rely
/// on the webview window being destroyed (which drops all its listeners)
/// rather than storing the handle. Future work: store in a global map.
pub(super) async fn remove_webview_message_listener_impl(_env:&MountainEnvironment, handle:&str) {
	dev_log!(
		"extensions",
		"[WebviewProvider] Message listener unregistered for handle={}",
		handle
	);
}
