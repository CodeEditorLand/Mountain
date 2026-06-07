//! Sky dispatcher - handles sky:* and editor:* commands.

use serde_json::Value;

use crate::{Sky::ReplayEvents::Fn as SkyReplayEvents, Utilities::JsonValueHelpers::arg_val};

/// Dispatches Sky commands.
///
/// Handled commands:
/// - `sky:replay-events`
/// - `editor:revealRange` / `window:revealRange`
/// - `sky:editor:selectionChanged`
/// - `sky:model:contentChanged`
/// - `sky:editor:activeChanged`
/// - `sky:editor:visibleChanged`
/// - `sky:editor:tabsChanged`
/// - `sky:editor:visibleRangesChanged`
/// - `sky:editor:optionsChanged`
/// - `sky:editor:diffInformationChanged`
/// - `sky:editor:viewColumnChanged`
pub async fn dispatch_sky(
	app_handle:&tauri::AppHandle,

	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"sky:replay-events" => SkyReplayEvents(app_handle.clone(), runtime.clone()).await,

		"editor:revealRange" | "window:revealRange" => {
			let payload = arg_val(&arguments, 0);

			let _ = app_handle.emit("sky://editor/revealRange", &payload);

			Ok(Value::Null)
		},

		"sky:editor:selectionChanged" => {
			let uri = arguments
				.first()
				.and_then(|v| v.get("uri"))
				.and_then(|v| v.as_str())
				.unwrap_or("")
				.to_string();

			let selections = arguments
				.first()
				.and_then(|v| v.get("selections"))
				.cloned()
				.unwrap_or(Value::Array(Vec::new()));

			let view_column = arguments
				.first()
				.and_then(|v| v.get("viewColumn"))
				.and_then(|v| v.as_u64())
				.unwrap_or(1);

			if !uri.is_empty() {
				runtime
					.Environment
					.ApplicationState
					.Workspace
					.SetActiveDocumentURI(Some(uri.clone()));
			}

			let payload = json!({ "uri": uri, "selections": selections, "viewColumn": view_column });

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"window.didChangeTextEditorSelection".to_string(),
				payload,
			)
			.await;

			Ok(Value::Null)
		},

		"sky:model:contentChanged" => {
			// Forward to Cocoon
			let payload = arg_val(&arguments, 0);

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptModelChanged".to_string(),
				payload,
			)
			.await;

			Ok(Value::Null)
		},

		"sky:editor:activeChanged" => {
			let payload = arg_val(&arguments, 0);

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"window.didChangeActiveTextEditor".to_string(),
				payload,
			)
			.await;

			Ok(Value::Null)
		},

		"sky:editor:visibleChanged"
		| "sky:editor:tabsChanged"
		| "sky:editor:visibleRangesChanged"
		| "sky:editor:optionsChanged"
		| "sky:editor:diffInformationChanged"
		| "sky:editor:viewColumnChanged" => {
			let method = match command {
				"sky:editor:visibleChanged" => "$acceptVisibleEditorsChanged",

				"sky:editor:tabsChanged" => "$acceptTabsChanged",

				"sky:editor:visibleRangesChanged" => "$acceptVisibleRangesChanged",

				"sky:editor:optionsChanged" => "$acceptTextEditorOptionsChanged",

				"sky:editor:diffInformationChanged" => "$acceptTextEditorDiffInformationChanged",

				_ => "$acceptTextEditorViewColumnChanged",
			};

			let payload = arg_val(&arguments, 0);

			let _ =
				crate::Vine::Client::SendNotification::Fn("cocoon-main".to_string(), method.to_string(), payload).await;

			Ok(Value::Null)
		},

		_ => Err(format!("Unknown sky command: {}", command)),
	}
}
