
//! Tauri command - notify the provider when a tree node is
//! expanded / collapsed.
//!
//! ## Stub
//!
//! Wire `OnTreeNodeExpanded` into `CommonTreeViewProvider` and
//! dispatch here so providers can lazily load child items or persist
//! expansion state across sessions.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, State, Wry, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn OnTreeViewExpansionChanged(
	_ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	_ViewId:String,

	_ElementHandle:String,

	_IsExpanded:bool,
) -> Result<Value, String> {
	dev_log!("commands", "warn: OnTreeViewExpansionChanged not implemented");

	Ok(json!({ "success": false, "error": "OnTreeNodeExpanded method not implemented" }))
}
