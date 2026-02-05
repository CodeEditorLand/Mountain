//! # Tree View Event Handlers
//!
//! Internal helper functions for handling user interaction events
//! (expansion, selection).

use CommonLibrary::Error::CommonError::CommonError;
use log::info;
use serde_json::json;
use tauri::Emitter;

use crate::Environment::Utility;

/// Handles tree node expansion/collapse events.
/// Called when a user expands or collapses a node in the tree view.
pub(super) async fn on_tree_node_expanded(
	env: &crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier: String,
	element_handle: String,
	is_expanded: bool,
) -> Result<(), CommonError> {
	info!(
		"[TreeViewProvider] Node '{}' in view '{}' expanded: {}",
		element_handle, view_identifier, is_expanded
	);

	// Persist expansion state in TreeViewStateDTO for state restoration

	// Propagate to frontend
	env.ApplicationHandle
		.emit(
			"sky://tree-view/node-expanded",
			json!({
				"ViewIdentifier": view_identifier,
				"ElementHandle": element_handle,
				"IsExpanded": is_expanded
			}),
		)
		.map_err(|Error| {
			CommonError::UserInterfaceInteraction {
				Reason: format!("Failed to emit node expanded event: {}", Error),
			}
		})
}

/// Handles tree selection changes.
/// Called when the user selects or deselects items in the tree view.
pub(super) async fn on_tree_selection_changed(
	env: &crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier: String,
	selected_handles: Vec<String>,
) -> Result<(), CommonError> {
	info!(
		"[TreeViewProvider] Selection changed in view '{}': {} items selected",
		view_identifier,
		selected_handles.len()
	);

	// Persist selection state in TreeViewStateDTO for state restoration

	// Propagate to frontend
	env.ApplicationHandle
		.emit(
			"sky://tree-view/selection-changed",
			json!({
				"ViewIdentifier": view_identifier,
				"SelectedHandles": selected_handles
			}),
		)
		.map_err(|Error| {
			CommonError::UserInterfaceInteraction {
				Reason: format!("Failed to emit selection changed event: {}", Error),
			}
		})
}
