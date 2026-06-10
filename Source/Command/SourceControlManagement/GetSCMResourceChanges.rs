//! Tauri command - list resources for a given SCM provider. The
//! resources map is keyed by `(group_id, …)`; we flatten across
//! groups and filter by `ProviderHandle == ProviderIdentifier`.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{State, command};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, dev_log};

#[command]
pub async fn GetSCMResourceChanges(
	State:State<'_, Arc<ApplicationState>>,

	ProviderIdentifier:String,
) -> Result<Value, String> {
	dev_log!("commands", "getting resource changes for provider: {}", ProviderIdentifier);

	let resources_map = State.Feature.Markers.SourceControlManagementResources.lock().clone();

	let provider_handle_u32 = ProviderIdentifier.parse::<u32>().unwrap_or(0);

	let ProviderResources:Vec<_> = resources_map
		.iter()
		.flat_map(|(_group_id, group_resources)| group_resources.values())
		.flat_map(|vec_resources| vec_resources.iter())
		.filter(|r| r.ProviderHandle == provider_handle_u32)
		.cloned()
		.collect();

	Ok(json!({ "resources": ProviderResources }))
}
