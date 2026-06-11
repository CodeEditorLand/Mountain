//! Restore webview panels persisted before the previous reload by asking
//! Cocoon to deserialize each entry stashed under `__webview_panel_state__`
//! in global storage, then clear the cache.

use std::sync::Arc;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(SideCarId:&str, Environment:&Arc<MountainEnvironment>) {
	// Webview panel restore: any panels persisted before the previous
	// reload landed in global storage under `__webview_panel_state__`.
	// Now that extensions are activated and their serializers are
	// re-registered, ask Cocoon to deserialize each entry. Failures are
	// per-panel - one broken serializer doesn't block the others.
	use CommonLibrary::Storage::StorageProvider::StorageProvider;

	const PANEL_STATE_KEY:&str = "__webview_panel_state__";

	if let Ok(Some(Stored)) = Environment.GetStorageValue(true, PANEL_STATE_KEY).await {
		if let Some(Entries) = Stored.as_array() {
			if !Entries.is_empty() {
				dev_log!(
					"cocoon",
					"[CocoonManagement] Restoring {} webview panel(s) from previous reload",
					Entries.len()
				);
			}

			for Entry in Entries {
				let ViewType = Entry.get("viewType").and_then(|V| V.as_str()).unwrap_or("");

				if ViewType.is_empty() {
					continue;
				}

				let State = Entry.get("state").cloned().unwrap_or(serde_json::Value::Null);

				let DeserializeMethod = "ExtHostWebviewPanels$deserializeWebviewPanel".to_string();

				if let Err(Error) = crate::Vine::Client::SendRequest::Fn(
					SideCarId,
					DeserializeMethod,
					serde_json::json!([ViewType, serde_json::Value::Null, State]),
					5_000,
				)
				.await
				{
					dev_log!(
						"cocoon",
						"warn: [CocoonManagement] deserializeWebviewPanel({}) failed: {:?}",
						ViewType,
						Error
					);
				}
			}
		}

		// Clear the cache so panels aren't re-restored on the NEXT
		// reload if the user didn't have them open this session.
		let _ = Environment.UpdateStorageValue(true, PANEL_STATE_KEY.to_string(), None).await;
	}
}
