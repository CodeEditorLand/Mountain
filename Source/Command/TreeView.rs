// ============================================================================
// File: Mountain/Source/Command/TreeView.rs
// ============================================================================
// # TreeView Commands
//! Defines the specific Tauri command handlers for TreeView data requests
//! that originate from the `Sky` frontend UI.
//!
//! ## Key Features:
//! - Tree data retrieval (children, tree items)
//! - Tree expansion state management
//! - Tree selection handling
//! - Drag and drop support
//! - Tree view UI interaction
//!
//! ## VSCode Reference:
//! - vs/workbench/api/browser/mainThreadTreeViews.ts
//! - vs/workbench/api/common/extHostTreeViews.ts
//! - vs/workbench/contrib/files/browser/explorerView.ts
// ============================================================================

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	TreeView::TreeViewProvider::TreeViewProvider as CommonTreeViewProvider,
};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry, command};

use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// A specific Tauri command handler for the UI to fetch the children of a tree
/// view node. This handler dispatches to the correct provider (native or
/// proxied).
#[command]
pub async fn GetTreeViewChildren(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	ViewId:String,

	ElementHandle:Option<String>,
) -> Result<Value, String> {
	log::debug!(
		"[DispatchLogic] Getting TreeView children for '{}', element: {:?}",
		ViewId,
		ElementHandle
	);

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let Environment:Arc<MountainEnvironment> = RunTime.Environment.clone();

	let TreeProvider:Arc<dyn CommonTreeViewProvider> = Environment.Require();

	match TreeProvider.GetChildren(ViewId.clone(), ElementHandle).await {
		Ok(Children) => Ok(json!(Children)),
		Err(Error) => {
			let ErrorMessage = format!("Failed to get children for tree view '{}': {}", ViewId, Error);
			log::error!("{}", ErrorMessage);
			Err(ErrorMessage)
		},
	}
}

/// Gets the tree item for a given tree element handle.
#[command]
pub async fn GetTreeViewItem(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	ViewId:String,

	ElementHandle:String,
) -> Result<Value, String> {
	log::debug!(
		"[DispatchLogic] Getting TreeView item for '{}', element: {}",
		ViewId,
		ElementHandle
	);

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let Environment:Arc<MountainEnvironment> = RunTime.Environment.clone();

	let TreeProvider:Arc<dyn CommonTreeViewProvider> = Environment.Require();

	match TreeProvider.GetTreeItem(ViewId.clone(), ElementHandle).await {
		Ok(Item) => Ok(json!(Item)),
		Err(Error) => {
			let ErrorMessage = format!("Failed to get tree item for view '{}': {}", ViewId, Error);
			log::error!("{}", ErrorMessage);
			Err(ErrorMessage)
		},
	}
}

/// Handles tree node expansion/collapse events.
/// TODO: OnTreeNodeExpanded is not defined in TreeViewProvider trait - needs implementation
#[command]
pub async fn OnTreeViewExpansionChanged(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	_ViewId:String,

	_ElementHandle:String,

	_IsExpanded:bool,
) -> Result<Value, String> {
	log::warn!("[TreeView Command] OnTreeViewExpansionChanged not implemented");

	Ok(json!({ "success": false, "error": "OnTreeNodeExpanded method not implemented" }))
}

/// Handles tree selection changes.
/// TODO: OnTreeSelectionChanged is not defined in TreeViewProvider trait - needs implementation
#[command]
pub async fn OnTreeViewSelectionChanged(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	_ViewId:String,

	_SelectedHandles:Vec<String>,
) -> Result<Value, String> {
	log::warn!("[TreeView Command] OnTreeViewSelectionChanged not implemented");

	Ok(json!({ "success": false, "error": "OnTreeSelectionChanged method not implemented" }))
}

/// Refreshes a tree view.
#[command]
pub async fn RefreshTreeView(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	ViewId:String,

	ItemsToRefresh:Option<Vec<String>>,
) -> Result<Value, String> {
	log::debug!(
		"[TreeView Command] Refreshing tree view '{}', items: {:?}",
		ViewId,
		ItemsToRefresh
	);

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let Environment:Arc<MountainEnvironment> = RunTime.Environment.clone();

	let RefreshValue:Option<Value> = ItemsToRefresh.and_then(|items| serde_json::to_value(items).ok());

	match Environment.RefreshTreeView(ViewId.clone(), RefreshValue).await {
		Ok(_) => Ok(json!({ "success": true })),
		Err(Error) => {
			let ErrorMessage = format!("Failed to refresh tree view '{}': {}", ViewId, Error);
			log::error!("{}", ErrorMessage);
			Err(ErrorMessage)
		},
	}
}

/// Reveals a specific tree item.
#[command]
pub async fn RevealTreeViewItem(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	ViewId:String,

	ItemHandle:String,

	Options:Option<Value>,
) -> Result<Value, String> {
	log::debug!("[TreeView Command] Revealing item '{}' in view '{}'", ItemHandle, ViewId);

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let Environment:Arc<MountainEnvironment> = RunTime.Environment.clone();

	let OptionsValue = Options.unwrap_or(json!({}));

	match Environment.RevealTreeItem(ViewId.clone(), ItemHandle, OptionsValue).await {
		Ok(_) => Ok(json!({ "success": true })),
		Err(Error) => {
			let ErrorMessage = format!("Failed to reveal tree item in view '{}': {}", ViewId, Error);
			log::error!("{}", ErrorMessage);
			Err(ErrorMessage)
		},
	}
}

/// Persists tree view state.
/// TODO: PersistTreeViewState is not defined in TreeViewProvider trait - needs implementation
#[command]
pub async fn PersistTreeView(
	_ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	_ViewId:String,
) -> Result<Value, String> {
	log::warn!("[TreeView Command] PersistTreeView not implemented");

	Ok(json!({ "success": false, "error": "PersistTreeViewState method not implemented" }))
}

/// Restores tree view state.
/// TODO: RestoreTreeViewState is not defined in TreeViewProvider trait - needs implementation
#[command]
pub async fn RestoreTreeView(
	_ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	_ViewId:String,

	_StateValue:Value,
) -> Result<Value, String> {
	log::warn!("[TreeView Command] RestoreTreeView not implemented");

	Ok(json!({ "success": false, "error": "RestoreTreeViewState method not implemented" }))
}
