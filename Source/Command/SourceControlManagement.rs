//! # SourceControlManagement Commands
//!
//! Defines the specific Tauri command handlers for SourceControlManagement data
//! requests that originate from the `Sky` frontend UI.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::ApplicationState::ApplicationState::{ApplicationState, MapLockError};

/// Retrieves the complete state of all Source Control Management providers,
/// groups, and resources for rendering in the UI.
///
/// This command is called by the frontend to get a full snapshot of the SCM
/// view.
#[command]
pub async fn GetAllSourceControlManagementState(State:State<'_, Arc<ApplicationState>>) -> Result<Value, String> {
	log::debug!("[SourceControlManagement Command] Getting all SCM state for UI.");

	let Providers = State
		.SourceControlManagementProviders
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	let Groups = State
		.SourceControlManagementGroups
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	let Resources = State
		.SourceControlManagementResources
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	Ok(json!({
		"providers": Providers,
		"groups": Groups,
		"resources": Resources,
	}))
}
