//! TreeView dispatcher.

use serde_json::Value;

use crate::TreeView::GetChildren::Fn as TreeGetChildren;

/// Dispatches tree view commands.
pub async fn dispatch_tree_view(
	app_handle:&tauri::AppHandle,

	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"tree:getChildren" => TreeGetChildren(app_handle.clone(), runtime.clone(), arguments).await,

		"tree.reveal" | "tree:reveal" => {
			let view_id = crate::Utilities::JsonValueHelpers::arg_string(&arguments, 0);

			let handle = crate::Utilities::JsonValueHelpers::arg_string(&arguments, 1);

			let options = crate::Utilities::JsonValueHelpers::arg_val(&arguments, 2);

			crate::dev_log!("ipc", "tree.reveal viewId={} handle={}", view_id, handle);

			let _ = app_handle.emit(
				"sky://tree-view/reveal",
				serde_json::json!({
					"viewId": view_id,
					"handle": handle,
					"options": options,
				}),
			);

			Ok(Value::Null)
		},

		"tree:selectionChanged" | "tree:collapseElement" | "tree:expandElement" | "tree:visibilityChanged" => {
			let payload = crate::Utilities::JsonValueHelpers::arg_val(&arguments, 0);

			let method = match command {
				"tree:selectionChanged" => "$treeView:selectionChanged",

				"tree:collapseElement" => "$treeView:collapseElement",

				"tree:expandElement" => "$treeView:expandElement",

				_ => "$treeView:visibilityChanged",
			};

			tokio::spawn(async move {
				if let Err(e) =
					crate::Vine::Client::SendNotification::Fn("cocoon-main".to_string(), method.to_string(), payload)
						.await
				{
					crate::dev_log!("ipc", "warn: [tree] Cocoon notify {} failed: {:?}", method, e);
				}
			});

			Ok(Value::Null)
		},

		_ => Err(format!("Unknown tree view command: {}", command)),
	}
}
