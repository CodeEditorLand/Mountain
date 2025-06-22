//! # SourceControlManagement Commands
//!
//! Defines the specific Tauri command handlers for SourceControlManagement data requests
//! that originate from the `Sky` frontend UI.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::ApplicationState::ApplicationState::ApplicationState;

#[command]
pub async fn GetAllSourceControlManagementState(state:State<'_, Arc<ApplicationState>>) -> Result<Value, String> {
	log::debug!("[SourceControlManagement Command] Getting all SourceControlManagement state for UI.");

	let providers = state.SourceControlManagementProviders.lock().unwrap().clone();
	let groups = state.SourceControlManagementGroups.lock().unwrap().clone();
	let resources = state.SourceControlManagementResources.lock().unwrap().clone();

	Ok(json!({
		"providers": providers,
		"groups": groups,
		"resources": resources
	}))
}
