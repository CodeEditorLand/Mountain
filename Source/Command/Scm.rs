//! # SCM Commands
//!
//! Defines the specific Tauri command handlers for SCM data requests
//! that originate from the `Sky` frontend UI.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::ApplicationState::ApplicationState::ApplicationState;

#[command]
pub async fn GetAllScmState(state:State<'_, Arc<ApplicationState>>) -> Result<Value, String> {
	log::debug!("[SCM Command] Getting all SCM state for UI.");

	let providers = state.ScmProviders.lock().unwrap().clone();
	let groups = state.ScmGroups.lock().unwrap().clone();
	let resources = state.ScmResources.lock().unwrap().clone();

	Ok(json!({
		"providers": providers,
		"groups": groups,
		"resources": resources
	}))
}
