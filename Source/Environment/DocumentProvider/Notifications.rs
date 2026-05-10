//! Document lifecycle notification helpers.
//!
//! Sends LSP-style RPC notifications to the Cocoon extension-host sidecar
//! so its `ExtHostDocuments` model stays in sync with Mountain's
//! `OpenDocuments` state. All four helpers follow the same pattern:
//! serialize a payload, call `IPCProvider::SendNotificationToSideCar`, and
//! log any emit error without propagating it (notifications are fire-and-forget).
//!
//! | Helper | Cocoon RPC method | Trigger |
//! |---|---|---|
//! | `notify_model_added` | `$acceptModelAdded` | document opened / created |
//! | `notify_model_changed` | `$acceptModelChanged` | incremental text edit applied |
//! | `notify_model_saved` | `$acceptModelSaved` | document written to disk |
//! | `notify_model_removed` | `$acceptModelRemoved` | document closed or renamed |

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, IPC::IPCProvider::IPCProvider};
use serde_json::json;
use url::Url;

use crate::dev_log;

/// Notifies Cocoon that a new document model has been added.
pub(super) async fn notify_model_added(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_state_dto:&serde_json::Value,
) {
	let uri_string = document_state_dto
		.get("URI")
		.and_then(serde_json::Value::as_str)
		.unwrap_or("unknown");

	dev_log!("model", "[DocumentProvider] Notifying ModelAdded for: {}", uri_string);

	let payload = json!([document_state_dto]);

	let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

	if let Err(error) = ipc_provider
		.SendNotificationToSideCar("cocoon-main".to_string(), "$acceptModelAdded".to_string(), payload)
		.await
	{
		dev_log!(
			"model",
			"error: [DocumentProvider] Failed to send $acceptModelAdded for {}: {}",
			uri_string,
			error
		);
	}
}

/// Notifies Cocoon that a document's content has changed.
pub(super) async fn notify_model_changed(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	uri:&Url,

	new_version:i64,

	changes:serde_json::Value,
) {
	dev_log!("model", "[DocumentProvider] Notifying ModelChanged for: {}", uri);

	let uri_components = json!({ "external": uri.to_string(), "$mid": 1 });

	let event_data = json!({ "versionId": new_version, "changes": changes, "isDirty": true });

	let payload = json!([uri_components, event_data]);

	let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

	if let Err(error) = ipc_provider
		.SendNotificationToSideCar("cocoon-main".to_string(), "$acceptModelChanged".to_string(), payload)
		.await
	{
		dev_log!(
			"model",
			"error: [DocumentProvider] Failed to send $acceptModelChanged for {}: {}",
			uri,
			error
		);
	}
}

/// Notifies Cocoon that a document has been saved to disk.
pub(super) async fn notify_model_saved(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	uri:&Url,
) {
	dev_log!("model", "[DocumentProvider] Notifying ModelSaved for: {}", uri);

	let uri_components = json!({ "external": uri.to_string(), "$mid": 1 });

	let payload = json!([uri_components]);

	let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

	if let Err(error) = ipc_provider
		.SendNotificationToSideCar("cocoon-main".to_string(), "$acceptModelSaved".to_string(), payload)
		.await
	{
		dev_log!(
			"model",
			"error: [DocumentProvider] Failed to send $acceptModelSaved for {}: {}",
			uri,
			error
		);
	}
}

/// Notifies Cocoon that a document has been closed or renamed.
pub(super) async fn notify_model_removed(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	uri:&Url,
) {
	dev_log!("model", "[DocumentProvider] Notifying ModelRemoved for: {}", uri);

	let uri_components = json!({ "external": uri.to_string(), "$mid": 1 });

	let payload = json!([uri_components]);

	let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

	if let Err(error) = ipc_provider
		.SendNotificationToSideCar("cocoon-main".to_string(), "$acceptModelRemoved".to_string(), payload)
		.await
	{
		dev_log!(
			"model",
			"error: [DocumentProvider] Failed to send $acceptModelRemoved for {}: {}",
			uri,
			error
		);
	}
}
