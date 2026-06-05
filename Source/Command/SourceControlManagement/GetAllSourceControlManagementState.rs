//! Tauri command - full snapshot of every registered SCM provider,
//! its resource groups, and the resources within those groups.
//! Drives the SCM viewlet's first paint.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::{
	ApplicationState::State::ApplicationState::ApplicationState,
	dev_log,
};

#[command]
pub async fn GetAllSourceControlManagementState(State:State<'_, Arc<ApplicationState>>) -> Result<Value, String> {
	dev_log!("commands", "getting all SCM state for UI");

	let Providers = State
		.Feature
		.Markers
		.SourceControlManagementProviders
		.lock()
		.clone();

	let Groups = State
		.Feature
		.Markers
		.SourceControlManagementGroups
		.lock()
		.clone();

	let Resources = State
		.Feature
		.Markers
		.SourceControlManagementResources
		.lock()
		.clone();

	Ok(json!({
		"providers": Providers,
		"groups": Groups,
		"resources": Resources,
	}))
}
