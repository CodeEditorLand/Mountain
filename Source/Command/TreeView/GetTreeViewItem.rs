//! Tauri command - fetch a single tree item's metadata (label, icon,
//! description, command, contextValue) by its element handle.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	TreeView::TreeViewProvider::TreeViewProvider as CommonTreeViewProvider,
};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry, command};

use crate::{
	ApplicationState::State::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

#[command]
/// Gets tree view item.
pub async fn GetTreeViewItem(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	ViewId:String,

	ElementHandle:String,
) -> Result<Value, String> {
	dev_log!("commands", "getting TreeView item for '{}', element: {}", ViewId, ElementHandle);

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let Environment:Arc<MountainEnvironment> = RunTime.Environment.clone();

	let TreeProvider:Arc<dyn CommonTreeViewProvider> = Environment.Require();

	match TreeProvider.GetTreeItem(ViewId.clone(), ElementHandle).await {
		Ok(Item) => Ok(json!(Item)),

		Err(Error) => {
			let ErrorMessage = format!("Failed to get tree item for view '{}': {}", ViewId, Error);

			dev_log!("commands", "error: {}", ErrorMessage);

			Err(ErrorMessage)
		},
	}
}
