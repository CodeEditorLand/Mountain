//! Tauri command - focus / scroll-into-view a specific tree item.
//! `Options` carries the LSP-shaped `select`, `focus`, `expand`
//! booleans (matches `vscode.TreeView.reveal`).

use std::sync::Arc;

use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry, command};

use crate::{
	ApplicationState::Struct::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

#[command]
pub async fn Fn(
	ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	ViewId:String,

	ItemHandle:String,

	Options:Option<Value>,
) -> Result<Value, String> {
	dev_log!("commands", "revealing item '{}' in view '{}'", ItemHandle, ViewId);

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let Environment:Arc<MountainEnvironment> = RunTime.Environment.clone();

	let OptionsValue = Options.unwrap_or(json!({}));

	match Environment.RevealTreeItem(ViewId.clone(), ItemHandle, OptionsValue).await {
		Ok(_) => Ok(json!({ "success": true })),

		Err(Error) => {
			let ErrorMessage = format!("Failed to reveal tree item in view '{}': {}", ViewId, Error);

			dev_log!("commands", "error: {}", ErrorMessage);

			Err(ErrorMessage)
		},
	}
}
