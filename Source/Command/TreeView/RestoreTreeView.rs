#![allow(non_snake_case)]

//! Tauri command - deserialise + apply tree-view state captured by
//! `PersistTreeView` (sibling). Called when a tree view is recreated
//! or the workspace is reloaded.
//!
//! TODO: stub. Wire `RestoreTreeViewState` into the
//! `CommonTreeViewProvider` trait.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, State, Wry, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn RestoreTreeView(
	_ApplicationHandle:AppHandle<Wry>,
	_State:State<'_, Arc<ApplicationState>>,
	_ViewId:String,
	_StateValue:Value,
) -> Result<Value, String> {
	dev_log!("commands", "warn: RestoreTreeView not implemented");
	Ok(json!({ "success": false, "error": "RestoreTreeViewState method not implemented" }))
}
