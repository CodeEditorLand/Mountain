//! # Tree View Event Handlers
//!
//! Internal helper functions for handling user interaction events
//! (expansion, selection).

use CommonLibrary::{Error::CommonError::CommonError, IPC::SkyEvent::SkyEvent};
use serde_json::json;
use tauri::Emitter;

use crate::dev_log;

/// Handles tree node expansion/collapse events.
/// Called when a user expands or collapses a node in the tree view.
pub(super) async fn on_tree_node_expanded(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier:String,
	element_handle:String,
	is_expanded:bool,
) -> Result<(), CommonError> {
	dev_log!(
		"extensions",
		"[TreeViewProvider] Node '{}' in view '{}' expanded: {}",
		element_handle,
		view_identifier,
		is_expanded
	);

	// Persist expansion state in TreeViewStateDTO for state restoration

	// Propagate to frontend
	env.ApplicationHandle
		.emit(
			SkyEvent::TreeViewNodeExpanded.AsStr(),
			json!({
				"viewId": view_identifier,
				"elementHandle": element_handle,
				"expanded": is_expanded
			}),
		)
		.map_err(|Error| {
			CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit node expanded event: {}", Error) }
		})
}

/// Handles tree selection changes.
/// Called when the user selects or deselects items in the tree view.
pub(super) async fn on_tree_selection_changed(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier:String,
	selected_handles:Vec<String>,
) -> Result<(), CommonError> {
	dev_log!(
		"extensions",
		"[TreeViewProvider] Selection changed in view '{}': {} items selected",
		view_identifier,
		selected_handles.len()
	);

	// Persist selection state in TreeViewStateDTO for state restoration

	// Propagate to frontend
	env.ApplicationHandle
		.emit(
			SkyEvent::TreeViewSelectionChanged.AsStr(),
			json!({
				"viewId": view_identifier,
				"selectedHandles": selected_handles
			}),
		)
		.map_err(|Error| {
			CommonError::UserInterfaceInteraction {
				Reason:format!("Failed to emit selection changed event: {}", Error),
			}
		})
}
