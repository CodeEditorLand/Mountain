//! Tauri command - full snapshot of every registered SCM provider,
//! its resource groups, and the resources within those groups.
//! Drives the SCM viewlet's first paint.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{Struct, command};

use crate::{
	ApplicationState::Struct::ApplicationState::{ApplicationState, MapLockError},
	dev_log,
};

#[command]
pub async fn Fn(State:State<'_, Arc<ApplicationState>>) -> Result<Value, String> {
	dev_log!("commands", "getting all SCM state for UI");

	let Providers = State
		.Feature
		.Markers
		.SourceControlManagementProviders
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	let Groups = State
		.Feature
		.Markers
		.SourceControlManagementGroups
		.lock()
		.map_err(MapLockError)
		.map_err(|Error| Error.to_string())?
		.clone();

	let Resources = State
		.Feature
		.Markers
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
