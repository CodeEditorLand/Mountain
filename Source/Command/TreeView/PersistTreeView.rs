#![allow(non_snake_case)]

//! Tauri command - serialise tree-view state (expansion, selection,
//! scroll position) for cross-session restore.
//!
//! TODO: stub. Wire `PersistTreeViewState` into the
//! `CommonTreeViewProvider` trait; persist to workspace storage or
//! `ApplicationState` so `RestoreTreeView` (sibling) can reapply.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, State, Wry, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn PersistTreeView(
	_ApplicationHandle:AppHandle<Wry>,
	_State:State<'_, Arc<ApplicationState>>,
	_ViewId:String,
) -> Result<Value, String> {
	dev_log!("commands", "warn: PersistTreeView not implemented");
	Ok(json!({ "success": false, "error": "PersistTreeViewState method not implemented" }))
}
