
//! Tauri command - dispatch SCM operations (commit / push / pull).
//!
//! ## Stub
//!
//! Route through the `SourceControlManagementProvider` trait
//! instead of the inline match. Real provider invocation gives us
//! progress reporting, cancellation, and proper error surfacing.
//! Current shape returns mocked success.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn ExecuteSCMCommand(
	_State:State<'_, Arc<ApplicationState>>,

	CommandName:String,

	_Arguments:Value,
) -> Result<Value, String> {
	dev_log!("commands", "executing command: {}", CommandName);

	match CommandName.as_str() {
		"git.commit" | "commit" => {
			dev_log!("commands", "executing commit");

			Ok(json!({ "success": true, "message": "Commit successful" }))
		},

		"git.push" | "push" => {
			dev_log!("commands", "executing push");

			Ok(json!({ "success": true, "message": "Push successful" }))
		},

		"git.pull" | "pull" => {
			dev_log!("commands", "executing pull");

			Ok(json!({ "success": true, "message": "Pull successful" }))
		},

		_ => Err(format!("Unknown SCM command: {}", CommandName)),
	}
}
