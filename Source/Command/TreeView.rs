//! # TreeView Commands
//!
//! Defines the specific Tauri command handlers for TreeView data requests
//! that originate from the `Sky` frontend UI.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, State, command};

use crate::ApplicationState::ApplicationState::ApplicationState;

/// A specific Tauri command handler for the UI to fetch the children of a
/// tree view node.
#[command]
pub async fn GetTreeViewChildren(
	_AppicationHandle:AppHandle,

	state:State<'_, Arc<ApplicationState>>,

	view_id:String,

	element_handle:Option<String>,
) -> Result<Value, String> {
	log::debug!(
		"[DispatchLogic] Getting TreeView children for '{}', element: {:?}",
		view_id,
		element_handle
	);

	let provider = {
		let tree_views = state.ActiveTreeViews.lock().map_err(|e| e.to_string())?;

		// Note: This logic for getting a provider needs to be more robust.
		// Assuming a single native provider for now.
		tree_views.get(&view_id).and_then(|v| v.Provider.clone())
	};

	if let Some(provider) = provider {
		match provider.GetChildren(view_id, element_handle).await {
			Ok(children) => Ok(json!(children)),

			Err(e) => {
				let err_msg = format!("Failed to get children for tree view: {}", e);

				log::error!("{}", err_msg);

				Err(err_msg)
			},
		}
	} else {
		let err_msg = format!("No provider found for tree view '{}'", view_id);

		log::error!("{}", err_msg);

		Err(err_msg)
	}
}
