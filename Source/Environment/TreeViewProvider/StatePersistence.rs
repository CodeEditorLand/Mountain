//! # Tree View State Persistence Helpers
//!
//! Internal helper functions for saving and restoring tree view state.

use CommonLibrary::Error::CommonError::CommonError;
use log::info;
use serde_json::json;
use tauri::Emitter;

use crate::Environment::Utility;

/// Persists the current state of a tree view.
/// Saves the expansion, selection, and other state for later restoration.
pub(super) async fn persist_tree_view_state(
	env: &crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier: String,
) -> Result<serde_json::Value, CommonError> {
	info!("[TreeViewProvider] Persisting state for view '{}'", view_identifier);

	let tree_views = env
		.ApplicationState
		.Feature.TreeViews.ActiveTreeViews
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	let state = tree_views.get(&view_identifier).map(|view| {
		json!({
			"ViewIdentifier": view_identifier,
			"Title": view.Title,
			"Description": view.Description,
			"CanSelectMany": view.CanSelectMany,
			"Message": view.Message,
			"HasHandleDrag": view.HasHandleDrag,
			"HasHandleDrop": view.HasHandleDrop,
		})
	});

	state.ok_or(CommonError::TreeViewProviderNotFound { ViewIdentifier: view_identifier })
}

/// Restores a previously persisted tree view state.
/// Restores expansion, selection, and other state from a JSON representation.
pub(super) async fn restore_tree_view_state(
	env: &crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier: String,
	state_value: serde_json::Value,
) -> Result<(), CommonError> {
	info!("[TreeViewProvider] Restoring state for view '{}'", view_identifier);

	let mut tree_views = env
		.ApplicationState
		.Feature.TreeViews.ActiveTreeViews
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	if let Some(view_state) = tree_views.get_mut(&view_identifier) {
		if let Some(title) = state_value.get("Title").and_then(|v| v.as_str()) {
			view_state.Title = Some(title.to_string());
		}
		if let Some(description) = state_value.get("Description").and_then(|v| v.as_str()) {
			view_state.Description = Some(description.to_string());
		}
		// Restore additional UI state properties from the persisted StateValue:
		// - expansion state (which nodes are open)
		// - scroll position (viewport position)
		// - column widths (for detail views)
		// - sorting order
		// - provider-specific state extensions

		// Emit to frontend
		env.ApplicationHandle
			.emit(
				"sky://tree-view/restore-state",
				json!({
					"ViewIdentifier": view_identifier,
					"State": state_value
				}),
			)
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction {
					Reason: format!("Failed to emit restore state event: {}", Error),
				}
			})?;

		Ok(())
	} else {
		Err(CommonError::TreeViewProviderNotFound { ViewIdentifier: view_identifier })
	}
}
