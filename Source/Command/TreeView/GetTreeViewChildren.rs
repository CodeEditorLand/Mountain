//! Tauri command - fetch children for a tree node. `ElementHandle =
//! None` returns the root level. Dispatches through
//! `MountainEnvironment::Require<dyn TreeViewProvider>`.

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
pub async fn GetTreeViewChildren(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	ViewId:String,

	ElementHandle:Option<String>,
) -> Result<Value, String> {

	dev_log!(
		"commands",

		"getting TreeView children for '{}', element: {:?}",

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

			dev_log!("commands", "error: {}", ErrorMessage);

			Err(ErrorMessage)
		},
	}
}
