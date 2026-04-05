//! # Tree View Registration Helpers
//!
//! Internal helper functions for tree view provider registration and lifecycle.

use CommonLibrary::{Error::CommonError::CommonError, TreeView::DTO::TreeViewOptionsDTO::TreeViewOptionsDTO};
use log::info;
use serde_json::json;
use tauri::Emitter;

use crate::{ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO, Environment::Utility};

/// Registers a new tree data provider.
pub(super) async fn register_tree_data_provider(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier:String,
	options:serde_json::Value,
) -> Result<(), CommonError> {
	info!("[TreeViewProvider] Registering data provider for view: {}", view_identifier);

	let options_dto:TreeViewOptionsDTO = serde_json::from_value(options.clone())
		.map_err(|error| CommonError::InvalidArgument { ArgumentName:"Options".into(), Reason:error.to_string() })?;

	// For now, assume all extension providers come from the main sidecar.
	let side_car_identifier = "cocoon-main".to_string();

	let new_state = TreeViewStateDTO {
		ViewIdentifier:view_identifier.clone(),
		Provider:None,
		SideCarIdentifier:Some(side_car_identifier),
		CanSelectMany:options_dto.CanSelectMany,
		HasHandleDrag:options_dto.HasHandleDrag,
		HasHandleDrop:options_dto.HasHandleDrop,
		Message:None,
		Title:None,
		Description:None,
		Badge:None,
	};

	env.ApplicationState
		.Feature
		.TreeViews
		.ActiveTreeViews
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.insert(view_identifier.clone(), new_state);

	env.ApplicationHandle
		.emit(
			"sky://tree-view/create",
			json!({ "ViewIdentifier": view_identifier, "Options": options }),
		)
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;

	Ok(())
}

/// Unregisters a tree data provider.
pub(super) async fn unregister_tree_data_provider(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	view_identifier:String,
) -> Result<(), CommonError> {
	info!("[TreeViewProvider] Unregistering data provider for view: {}", view_identifier);

	env.ApplicationState
		.Feature
		.TreeViews
		.ActiveTreeViews
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.remove(&view_identifier);

	env.ApplicationHandle
		.emit("sky://tree-view/dispose", json!({ "ViewIdentifier": view_identifier }))
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
}
