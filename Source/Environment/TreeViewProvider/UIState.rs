//! # Tree View UI State Helpers
//!
//! Internal helper functions for updating tree view UI properties
//! (message, title, badge).

use CommonLibrary::{Error::CommonError::CommonError, IPC::SkyEvent::SkyEvent};
use serde_json::json;
use tauri::Emitter;

use crate::{Environment::Utility, dev_log};

/// Updates the tree view message displayed in the UI.
pub(super) async fn set_tree_view_message(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,

	view_identifier:String,

	message:Option<String>,
) -> Result<(), CommonError> {
	dev_log!(
		"extensions",
		"[TreeViewProvider] Setting message for view '{}': {:?}",
		view_identifier,
		message
	);

	{
		let mut tree_view_guard = env
			.ApplicationState
			.Feature
			.TreeViews
			.ActiveTreeViews
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		if let Some(view_state) = tree_view_guard.get_mut(&view_identifier) {
			view_state.Message = message.clone();
		}
	}

	env.ApplicationHandle
		.emit(
			SkyEvent::TreeViewSetMessage.AsStr(),
			json!({ "viewId": view_identifier, "message": message }),
		)
		.map_err(|Error| {
			CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit tree view message: {}", Error) }
		})
}

/// Updates the tree view's title and description.
pub(super) async fn set_tree_view_title(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,

	view_identifier:String,

	title:Option<String>,

	description:Option<String>,
) -> Result<(), CommonError> {
	dev_log!(
		"extensions",
		"[TreeViewProvider] Setting title/description for view '{}': {:?} {:?}",
		view_identifier,
		title,
		description
	);

	{
		let mut tree_view_guard = env
			.ApplicationState
			.Feature
			.TreeViews
			.ActiveTreeViews
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		if let Some(view_state) = tree_view_guard.get_mut(&view_identifier) {
			view_state.Title = title.clone();

			view_state.Description = description.clone();
		}
	}

	env.ApplicationHandle
		.emit(
			SkyEvent::TreeViewSetTitle.AsStr(),
			json!({
				"viewId": view_identifier,
				"title": title,
				"description": description,
			}),
		)
		.map_err(|Error| {
			CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit tree view title: {}", Error) }
		})
}

/// Sets a badge on the tree view.
pub(super) async fn set_tree_view_badge(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,

	view_identifier:String,

	badge:Option<serde_json::Value>,
) -> Result<(), CommonError> {
	dev_log!(
		"extensions",
		"[TreeViewProvider] Setting badge for view '{}': {:?}",
		view_identifier,
		badge
	);

	// Update state (badge field may need to be added to TreeViewStateDTO)
	{
		let mut tree_view_guard = env
			.ApplicationState
			.Feature
			.TreeViews
			.ActiveTreeViews
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		if let Some(view_state) = tree_view_guard.get_mut(&view_identifier) {
			// Store badge in ViewState
			if let Some(badge_value) = &badge {
				let badge_str = badge_value.to_string();

				if let Err(e) = view_state.SetBadge(badge_str) {
					dev_log!("extensions", "warn: Failed to set badge for view '{}': {}", view_identifier, e);
				}
			}
		}
	}

	// Emit to frontend
	env.ApplicationHandle
		.emit(
			SkyEvent::TreeViewSetBadge.AsStr(),
			json!({ "viewId": view_identifier, "badge": badge }),
		)
		.map_err(|Error| {
			CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit tree view badge: {}", Error) }
		})
}
