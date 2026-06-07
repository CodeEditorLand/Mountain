//! Tauri command - notify the provider when tree-item selection
//! changes (multi-select supported via `SelectedHandles`).
//!
//! ## Stub
//!
//! Wire `OnTreeSelectionChanged` into `CommonTreeViewProvider` so
//! providers can drive context-specific actions or detail-view updates.

use std::sync::Arc;

use serde_json::{Value, json};

use tauri::{AppHandle, State, Wry, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn OnTreeViewSelectionChanged(
	_ApplicationHandle:AppHandle<Wry>,

	_State:State<'_, Arc<ApplicationState>>,

	_ViewId:String,

	_SelectedHandles:Vec<String>,
) -> Result<Value, String> {

	dev_log!("commands", "warn: OnTreeViewSelectionChanged not implemented");

	Ok(json!({ "success": false, "error": "OnTreeSelectionChanged method not implemented" }))
}
