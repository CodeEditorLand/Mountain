//! # TreeView Commands
//!
//! Defines the specific Tauri command handlers for TreeView data requests
//! that originate from the `Sky` frontend UI.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Environment::Requires::Requires, TreeView::TreeViewProvider::TreeViewProvider as CommonTreeViewProvider};
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

	match TreeProvider.GetChildren(ViewId, ElementHandle).await {
		Ok(Children) => Ok(json!(Children)),

		Err(Error) => {
			let ErrorMessage = format!("Failed to get children for tree view: {}", Error);

			log::error!("{}", ErrorMessage);

			Err(ErrorMessage)
		},
	}
}
