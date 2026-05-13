#![allow(non_snake_case)]

//! Tauri command - stage / unstage a single resource. The standard
//! `git add` / `git restore --staged` flow.
//!
//! ## Stub
//!
//! Wire to `SourceControlManagementProvider::Stage` / `Unstage`.
//! Validate `ResourceURI` exists; support files and whole-directory
//! operations.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn StageSCMResource(
	_State:State<'_, Arc<ApplicationState>>,

	ResourceURI:String,

	Staged:bool,
) -> Result<Value, String> {
	dev_log!("commands", "staging resource: {}, staged: {}", ResourceURI, Staged);

	Ok(json!({ "success": true }))
}
