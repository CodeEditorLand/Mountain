//! Configuration value introspection.

use CommonLibrary::{
	Configuration::DTO::{
		ConfigurationOverridesDTO::ConfigurationOverridesDTO,
		InspectResultDataDTO::InspectResultDataDTO,
	},
	Error::CommonError::CommonError,
};
use log::info;
use serde_json::Value;
use tauri::Manager;

use crate::Environment::Utility;

/// Inspects a configuration key to get its value from all relevant scopes.
pub(super) async fn inspect_configuration_value(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	key: String,
	_overrides: ConfigurationOverridesDTO,
) -> Result<Option<InspectResultDataDTO>, CommonError> {
	info!("[ConfigurationProvider] Inspecting key: {}", key);

	let user_settings_path = environment
		.ApplicationHandle
		.path()
		.app_config_dir()
		.map(|p| p.join("settings.json"))
		.ok();

	let workspace_settings_path = environment
		.ApplicationState
		.Workspace.WorkspaceConfigurationPath
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.clone();

	// Read each configuration layer individually.
	let default_config = super::Loading::collect_default_configurations(&environment.ApplicationState)?;

	let user_config = super::Loading::read_and_parse_configuration_file(environment, &user_settings_path).await?;

	let workspace_config = super::Loading::read_and_parse_configuration_file(environment, &workspace_settings_path).await?;

	let get_value_from_dot_path = |node: &Value, path: &str| -> Option<Value> {
		path.split('.').try_fold(node, |n, k| n.get(k)).cloned()
	};

	let mut result_dto = InspectResultDataDTO::default();

	result_dto.DefaultValue = get_value_from_dot_path(&default_config, &key);

	result_dto.UserValue = get_value_from_dot_path(&user_config, &key);

	result_dto.WorkspaceValue = get_value_from_dot_path(&workspace_config, &key);

	// Determine the final effective value based on the correct cascade order.
	result_dto.EffectiveValue = result_dto
		.WorkspaceValue
		.clone()
		.or_else(|| result_dto.UserValue.clone())
		.or_else(|| result_dto.DefaultValue.clone());

	if result_dto.EffectiveValue.is_some() { Ok(Some(result_dto)) } else { Ok(None) }
}
