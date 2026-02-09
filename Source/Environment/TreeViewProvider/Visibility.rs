//! # Tree View Visibility Helpers
//!
//! Internal helper functions for tree view visibility and refresh operations.

use CommonLibrary::Error::CommonError::CommonError;
use log::info;
use serde_json::json;
use tauri::Emitter;

use crate::Environment::Utility;

/// Reveals a specific item in the tree view by notifying the UI.
pub(super) async fn reveal_tree_item(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier:String,
	item_handle:String,
	options:serde_json::Value,
) -> Result<(), CommonError> {
	info!(
		"[TreeViewProvider] Revealing item '{}' in view '{}'",
		item_handle, view_identifier
	);

	env.ApplicationHandle
		.emit(
			"sky://tree-view/reveal",
			json!({ "viewId": view_identifier, "itemHandle": item_handle, "options": options }),
		)
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
}

/// Refreshes the tree view by notifying the UI.
pub(super) async fn refresh_tree_view(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier:String,
	items_to_refresh:Option<serde_json::Value>,
) -> Result<(), CommonError> {
	info!("[TreeViewProvider] Refreshing view '{}'", view_identifier);

	env.ApplicationHandle
		.emit(
			"sky://tree-view/refresh",
			json!({ "viewId": view_identifier, "itemsToRefresh": items_to_refresh }),
		)
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
}
