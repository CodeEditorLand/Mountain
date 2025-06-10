use log::{error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};
use url::Url;

/// @module NotificationLogic (Documents/Handlers)
/// @description Contains the logic for sending document lifecycle notifications
/// from the Mountain host to the Cocoon sidecar, keeping the extension host's
/// state in sync.
use crate::{
	AppState::Dto::DocumentStateDto,
	vine::{self, client},
};

/// Notifies Cocoon that a new document model has been added (e.g., a file was
/// opened). @param DocumentStateDto - The DTO representing the initial state of
/// the new document.
pub async fn NotifyModelAdded<R:Runtime>(_AppHandle:&AppHandle<R>, DocumentStateDto:&Value) {
	let uri_str = DocumentStateDto.get("Uri").and_then(Value::as_str).unwrap_or("unknown");
	info!("[DocumentsNotify] Notifying ModelAdded for: {}", uri_str);

	// The payload is an array, as the corresponding handler in Cocoon expects it.
	let Payload = json!([DocumentStateDto]);

	if let Err(e) = client::SendNotification("cocoon-main", "$acceptModelAdded".to_string(), Payload).await {
		error!("[DocumentsNotify] Failed to send $acceptModelAdded for {}: {}", uri_str, e);
	}
}

/// Notifies Cocoon that a document's content has changed.
/// @param Uri - The URI of the document that changed.
/// @param NewVersion - The new version identifier for the document.
/// @param Changes - A DTO representing the text changes that occurred.
pub async fn NotifyModelChanged<R:Runtime>(_AppHandle:&AppHandle<R>, Uri:&Url, NewVersion:i64, Changes:Value) {
	info!("[DocumentsNotify] Notifying ModelChanged for: {}", Uri);

	// Construct the payload to match the VS Code protocol format.
	let UriComponents = json!({ "external": Uri.to_string(), "$mid": 1 });
	let EventData = json!({ "versionId": NewVersion, "changes": Changes });
	let Payload = json!([UriComponents, EventData, true]); // The final `true` is for `isDirty`.

	if let Err(e) = client::SendNotification("cocoon-main", "$acceptModelChanged".to_string(), Payload).await {
		error!("[DocumentsNotify] Failed to send $acceptModelChanged for {}: {}", Uri, e);
	}
}

/// Notifies Cocoon that a document has been saved to disk.
/// @param Uri - The URI of the saved document.
pub async fn NotifyModelSaved<R:Runtime>(_AppHandle:&AppHandle<R>, Uri:&Url) {
	info!("[DocumentsNotify] Notifying ModelSaved for: {}", Uri);

	let UriComponents = json!({ "external": Uri.to_string(), "$mid": 1 });
	let Payload = json!([UriComponents]);

	if let Err(e) = client::SendNotification("cocoon-main", "$acceptModelSaved".to_string(), Payload).await {
		error!("[DocumentsNotify] Failed to send $acceptModelSaved for {}: {}", Uri, e);
	}
}

/// Notifies Cocoon that a document has been closed.
/// @param Uri - The URI of the closed document.
pub async fn NotifyModelRemoved<R:Runtime>(_AppHandle:&AppHandle<R>, Uri:&Url) {
	info!("[DocumentsNotify] Notifying ModelRemoved for: {}", Uri);

	let UriComponents = json!({ "external": Uri.to_string(), "$mid": 1 });
	let Payload = json!([UriComponents]);

	if let Err(e) = client::SendNotification("cocoon-main", "$acceptModelRemoved".to_string(), Payload).await {
		error!("[DocumentsNotify] Failed to send $acceptModelRemoved for {}: {}", Uri, e);
	}
}
