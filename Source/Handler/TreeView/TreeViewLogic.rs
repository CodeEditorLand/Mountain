use Common::{error::CommonError, tree_view::dto::*};
use log::{error, info};
use serde_json::{Value, json};
use tauri::{ApplicationHandle, Emitter, Manager, RunTime, command};

// @module TreeViewLogic
// @description Contains the core logic for managing tree view state and
// orchestrating the data flow between the extension host (Cocoon) and the UI
// (Sky).
use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	ApplicationState::DTO::TreeViewStateDto,
	Handler::error_utils,
	vine::{self, client},
};

// Logic to register a new tree data provider from Cocoon. This is called by
// the `TreeViewProvider` in the environment.
pub async fn RegisterTreeDataProviderLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	ViewId:String,
	Options:TreeViewOptionsDto,
) -> Result<(), CommonError> {
	info!("[TreeViewLogic] Registering data provider for view: {}", ViewId);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();

	let NewState = TreeViewStateDto {
		ViewId:ViewId.clone(),
		CanSelectMany:Options.CanSelectMany,
		HasHandleDrag:Options.HasHandleDrag,
		HasHandleDrop:Options.HasHandleDrop,
	};

	AppStateInstance
		.ActiveTreeViews
		.lock()
		.unwrap()
		.insert(ViewId.clone(), NewState);

	// Notify the frontend to create the UI for this tree view.
	ApplicationHandle
		.emit("sky://tree-view/create", json!({ "ViewId": ViewId, "Options": Options }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to unregister a tree data provider.
pub async fn UnregisterTreeDataProviderLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	ViewId:String,
) -> Result<(), CommonError> {
	info!("[TreeViewLogic] Unregistering data provider for view: {}", ViewId);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	AppStateInstance.ActiveTreeViews.lock().unwrap().remove(&ViewId);
	ApplicationHandle
		.emit("sky://tree-view/dispose", json!({ "ViewId": ViewId }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// A Tauri command invoked by the Sky frontend when it needs the children of a
// tree node. This function acts as a proxy, fetching the data from the
// extension host via gRPC.
#[command]
pub async fn MountainRequestTreeChildren<R:RunTime>(
	ApplicationHandle:ApplicationHandle<R>,
	ViewId:String,
	ParentHandle:Option<String>,
) -> Result<Vec<TreeItemDto>, String> {
	info!(
		"[TreeViewLogic] UI requesting children for view '{}', parent: {:?}",
		ViewId, ParentHandle
	);

	let Response = client::SendRequest(
		"cocoon-main",
		"$getChildren".to_string(),
		json!({ "ViewId": ViewId, "ParentHandle": ParentHandle }),
		60000,
	)
	.await
	.map_err(|e| error_utils::MapCommonErrorToRpcString(e, "RequestTreeChildren"))?;

	serde_json::from_value(Response)
		.map_err(|e| error_utils::RpcErrorString(format!("Failed to deserialize TreeItemDto list: {}", e), None))
}

// Logic to reveal a specific item in the UI. This is called by the environment
// provider.
pub async fn RevealTreeItemLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	ViewId:String,
	Item:TreeItemDto,
	ParentChain:Vec<TreeItemDto>,
	Options:RevealOptionsDto,
) -> Result<(), CommonError> {
	info!("[TreeViewLogic] Revealing item in view: {}", ViewId);
	ApplicationHandle
		.emit(
			"sky://tree-view/reveal",
			json!({ "ViewId": ViewId, "Item": Item, "ParentChain": ParentChain, "Options": Options }),
		)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

// Logic to refresh a tree view. This is called by the environment provider
// when an extension fires the `onDidChangeTreeData` event.
pub async fn RefreshTreeViewLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	ViewId:String,
	ItemsToRefresh:Option<Value>,
) -> Result<(), CommonError> {
	info!("[TreeViewLogic] Refreshing view: {}", ViewId);
	ApplicationHandle
		.emit("sky://tree-view/refresh", json!({ "ViewId": ViewId, "Items": ItemsToRefresh }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
