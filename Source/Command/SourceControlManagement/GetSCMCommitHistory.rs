//! Tauri command - paginated commit log for the SCM viewlet's
//! Timeline panel.
//!
//! ## Stub
//!
//! Wire to `SourceControlManagementProvider::GetHistory`; return
//! structured commits with hash, author, date, message, parents.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn GetSCMCommitHistory(
	_State:State<'_, Arc<ApplicationState>>,

	MaxCount:Option<usize>,
) -> Result<Value, String> {
	dev_log!("commands", "getting commit history, max count: {:?}", MaxCount);

	let MaxCommits = MaxCount.unwrap_or(50);

	Ok(json!({
		"commits": Vec::<Value>::new(),
		"maxCount": MaxCommits,
	}))
}
