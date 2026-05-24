//! Tauri command - switch the working tree to a different branch.
//!
//! ## Stub
//!
//! Real implementation through `SourceControlManagementProvider::Checkout`
//! should handle uncommitted-changes prompts (stash / abort), branch creation
//! when missing, and upstream-tracking setup.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{Struct, command};

use crate::{ApplicationState::Struct::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn Fn(_State:State<'_, Arc<ApplicationState>>, BranchName:String) -> Result<Value, String> {
	dev_log!("commands", "checking out branch: {}", BranchName);

	Ok(json!({ "success": true, "message": format!("Checked out branch: {}", BranchName) }))
}
