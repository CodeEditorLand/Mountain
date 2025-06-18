// @module TreeViewLogic
// @description Contains the core logic for managing tree view state and
// orchestrating the data flow between the extension host (Cocoon) and the User Interface
// (Sky).

use Common::{error::CommonError, tree_view::DTO::*};
use log::{error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime, command};

use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::TreeViewStateDTO},
	Handler::error_utils,
	Vine::client,
};

// Logic to register a new tree data provider from Cocoon. This is called by
// the `TreeViewProvider` in the Environment.
pub async fn RegisterTreeDataProviderLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	view_id:String,
	options:TreeViewOptionsDTO,
) -> Result<(), CommonError> {
	info!("[TreeViewLogic] Registering data provider for view: {}", view_id);
	let app_state = app_handle.state::<ApplicationState>();

	let new_state = TreeViewStateDTO {
		ViewId:view_id.clone(),
		CanSelectMany:options.CanSelectMany,
		HasHandleDrag:options.HasHandleDrag,
		HasHandleDrop:options.HasHandleDrop,
	};

	app_state.ActiveTreeViews.lock().unwrap().insert(view_id.clone(), new_state);

	// Notify the frontend to create the User Interface for this tree view.
	app_handle
		.emit("sky://tree-view/create", json!({ "ViewId": view_id, "Options": options }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to unregister a tree data provider.
pub async fn UnregisterTreeDataProviderLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	view_id:String,
) -> Result<(), CommonError> {
	info!("[TreeViewLogic] Unregistering data provider for view: {}", view_id);
	let app_state = app_handle.state::<ApplicationState>();
	app_state.ActiveTreeViews.lock().unwrap().remove(&view_id);
	app_handle
		.emit("sky://tree-view/dispose", json!({ "ViewId": view_id }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// A Tauri command invoked by the Sky frontend when it needs the children of a
// tree node. This function acts as a proxy, fetching the data from the
// extension host via gRPC.
#[command]
pub async fn MountainRequestTreeChild<R:Runtime>(
	app_handle:AppHandle<R>,
	view_id:String,
	element_handle:Option<String>,
) -> Result<Vec<TreeItemDTO>, String> {
	info!(
		"[TreeViewLogic] User Interface requesting children for view '{}', element: {:?}",
		view_id, element_handle
	);

	let response = client::SendRequest(
		"cocoon-main",
		"$getChildren".to_string(),
		json!([view_id, element_handle]),
		60000,
	)
	.await
	.map_err(|e| error_utils::MapCommonErrorToRPCString(e, "RequestTreeChildren"))?;

	serde_json::from_value(response)
		.map_err(|e| error_utils::RPCErrorString(format!("Failed to deserialize TreeItemDTO list: {}", e), None))
}

// Logic to reveal a specific item in the User Interface. This is called by the Environment
// provider.
pub async fn RevealTreeItemLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	view_id:String,
	item:TreeItemDTO,
	parent_chain:Vec<TreeItemDTO>,
	options:RevealOptionsDTO,
) -> Result<(), CommonError> {
	info!("[TreeViewLogic] Revealing item in view: {}", view_id);
	app_handle
		.emit(
			"sky://tree-view/reveal",
			json!({ "ViewId": view_id, "Item": item, "ParentChain": parent_chain, "Options": options }),
		)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to refresh a tree view. This is called by the Environment provider
// when an extension fires the `onDidChangeTreeData` event.
pub async fn RefreshTreeViewLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	view_id:String,
	items_to_refresh:Option<Value>,
) -> Result<(), CommonError> {
	info!("[TreeViewLogic] Refreshing view: {}", view_id);
	app_handle
		.emit(
			"sky://tree-view/refresh",
			json!({ "ViewId": view_id, "Items": items_to_refresh }),
		)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to set a message on a tree view (e.g., "No results found").
pub async fn SetTreeViewMessageLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	view_id:String,
	message:Value,
) -> Result<(), CommonError> {
	app_handle
		.emit("sky://tree-view/set-message", json!({ "ViewId": view_id, "Message": message }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to set the title of a tree view.
pub async fn SetTreeViewTitleLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	view_id:String,
	title:String,
	description:Option<String>,
) -> Result<(), CommonError> {
	app_handle
		.emit(
			"sky://tree-view/set-title",
			json!({ "ViewId": view_id, "Title": title, "Description": description }),
		)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to set a badge on a tree view.
pub async fn SetTreeViewBadgeLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	view_id:String,
	badge:Option<TreeViewBadgeDTO>,
) -> Result<(), CommonError> {
	app_handle
		.emit("sky://tree-view/set-badge", json!({ "ViewId": view_id, "Badge": badge }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
