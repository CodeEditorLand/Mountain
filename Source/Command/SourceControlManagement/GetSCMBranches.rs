#![allow(non_snake_case)]

//! Tauri command - list branches for an SCM provider. Drives the
//! branch picker UI.
//!
//! ## Stub
//!
//! Wire to `SourceControlManagementProvider::GetBranches` so local
//! and remote branches with tracking relationships and current-branch
//! indicator are returned.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn GetSCMBranches(
	_State:State<'_, Arc<ApplicationState>>,

	ProviderIdentifier:String,
) -> Result<Value, String> {
	dev_log!("commands", "getting branches for provider: {}", ProviderIdentifier);

	Ok(json!({
		"branches": [
			{ "name": "main", "isCurrent": true },
			{ "name": "develop", "isCurrent": false },
		],
	}))
}
