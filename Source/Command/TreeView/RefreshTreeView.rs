//! Tauri command - request a tree view refresh, optionally targeting
//! specific item handles. `None` refreshes the entire tree.

use std::sync::Arc;

use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

use serde_json::{Value, json};

use tauri::{AppHandle, Manager, State, Wry, command};

use crate::{
	ApplicationState::State::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

#[command]
pub async fn RefreshTreeView(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	ViewId:String,

	ItemsToRefresh:Option<Vec<String>>,
) -> Result<Value, String> {

	dev_log!("commands", "refreshing tree view '{}', items: {:?}", ViewId, ItemsToRefresh);

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let Environment:Arc<MountainEnvironment> = RunTime.Environment.clone();

	let RefreshValue:Option<Value> = ItemsToRefresh.and_then(|items| serde_json::to_value(items).ok());

	match Environment.RefreshTreeView(ViewId.clone(), RefreshValue).await {
		Ok(_) => Ok(json!({ "success": true })),

		Err(Error) => {
			let ErrorMessage = format!("Failed to refresh tree view '{}': {}", ViewId, Error);

			dev_log!("commands", "error: {}", ErrorMessage);

			Err(ErrorMessage)
		},
	}
}
