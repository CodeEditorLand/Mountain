// @module NotificationLogic (document/Handler)
// @description Contains the logic for sending document lifecycle notifications
// from the Mountain host to the Cocoon sidecar, keeping the extension host's
// state in sync.

use log::{error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};
use url::Url;

use crate::Vine::client;

// Notifies Cocoon that a new document model has been added (e.g., a file was
// opened).
// @param document_state_DTO - The DTO representing the initial state of the
// new document.
pub async fn NotifyModelAdded<R:Runtime>(_app_handle:&AppHandle<R>, document_state_DTO:&Value) {
	let uri_str = document_state_DTO.get("Uri").and_then(Value::as_str).unwrap_or("unknown");
	info!("[DocumentNotify] Notifying ModelAdded for: {}", uri_str);

	// The payload is an array, as the corresponding handler in Cocoon expects it.
	let payload = json!([document_state_DTO]);

	if let Err(e) = client::SendNotification("cocoon-main", "$acceptModelAdded".to_string(), payload).await {
		error!("[DocumentNotify] Failed to send $acceptModelAdded for {}: {}", uri_str, e);
	}
}

// Notifies Cocoon that a document's content has changed.
// @param uri - The URI of the document that changed.
// @param new_version - The new version identifier for the document.
// @param changes - A DTO representing the text changes that occurred.
pub async fn NotifyModelChanged<R:Runtime>(_app_handle:&AppHandle<R>, uri:&Url, new_version:i64, changes:Value) {
	info!("[DocumentNotify] Notifying ModelChanged for: {}", uri);

	// Construct the payload to match the VS Code protocol format.
	let uri_components = json!({ "external": uri.to_string(), "$mid": 1 });
	let event_data = json!({ "versionId": new_version, "changes": changes });
	let payload = json!([uri_components, event_data, true]); // The final `true` is for `isDirty`.

	if let Err(e) = client::SendNotification("cocoon-main", "$acceptModelChanged".to_string(), payload).await {
		error!("[DocumentNotify] Failed to send $acceptModelChanged for {}: {}", uri, e);
	}
}

// Notifies Cocoon that a document has been saved to disk.
// @param uri - The URI of the saved document.
pub async fn NotifyModelSaved<R:Runtime>(_app_handle:&AppHandle<R>, uri:&Url) {
	info!("[DocumentNotify] Notifying ModelSaved for: {}", uri);

	let uri_components = json!({ "external": uri.to_string(), "$mid": 1 });
	let payload = json!([uri_components]);

	if let Err(e) = client::SendNotification("cocoon-main", "$acceptModelSaved".to_string(), payload).await {
		error!("[DocumentNotify] Failed to send $acceptModelSaved for {}: {}", uri, e);
	}
}

// Notifies Cocoon that a document has been closed.
// @param uri - The URI of the closed document.
pub async fn NotifyModelRemoved<R:Runtime>(_app_handle:&AppHandle<R>, uri:&Url) {
	info!("[DocumentNotify] Notifying ModelRemoved for: {}", uri);

	let uri_components = json!({ "external": uri.to_string(), "$mid": 1 });
	let payload = json!([uri_components]);

	if let Err(e) = client::SendNotification("cocoon-main", "$acceptModelRemoved".to_string(), payload).await {
		error!("[DocumentNotify] Failed to send $acceptModelRemoved for {}: {}", uri, e);
	}
}
