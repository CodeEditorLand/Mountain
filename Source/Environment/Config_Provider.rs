// ---------------------------------------------------------------------------------------------
// Mountain Environment - Configuration Provider
// 
// --------------------------------------------------------------------------------------------
// This module implements the `ConfigProvider` and `ConfigInspector` traits for
// `MountainEnvironment`. It handles retrieving effective configuration values,
// updating settings in various scopes (User, Workspace, WorkspaceFolder), and
// (partially) inspecting configuration value sources.
//
// It interacts closely with `AppState.configuration` (the merged view) and
// delegates persistence and complex merging logic to `handlers::config`.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	config_effects::{
		ConfigInspector,
		ConfigProvider,
		ConfigurationTarget,
		IConfigurationOverrides,
		InspectResultData,
	},
	environment::Requires,
	errors::CommonError,
};
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use serde_json::Value;

use crate::{
	app_state::AppState, // For accessing configuration state
	environment::{MountainEnvironment, utils::map_app_state_lock_error_to_common_error},
	handlers, // For config persistence and notification helpers
};

// --- ConfigProvider Implementation ---
#[async_trait]
impl ConfigProvider for MountainEnvironment {
	async fn get_configuration_value(
		&self,
		section_key_opt:Option<String>,    // e.g., "editor.fontSize" or None for all
		overrides:IConfigurationOverrides, // For resource/language-specific values
	) -> Result<Value, CommonError> {
		trace!(
			"[Env CfgProv] GetConfig: section={:?}, overrides.resource={:?}, overrides.langId={:?}",
			section_key_opt,
			overrides.resource.as_ref().and_then(|v| v.get("external")), // Log external URI
			overrides.override_identifier
		);

		let app_state = self.get_app_state();
		let config_state_guard = app_state
			.configuration
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		// `MergedConfigurationState::get_value` uses the pre-merged state.
		// TODO: Enhance this to fully respect `overrides` by potentially re-evaluating
		// against specific configuration files or layers if needed.
		if overrides.resource.is_some() || overrides.override_identifier.is_some() {
			warn!(
				"[Env CfgProv GetConfig] Overrides provided (resource or languageId), but current implementation \
				 primarily uses the pre-merged configuration state. Fine-grained override resolution beyond initial \
				 merge might be limited."
			);
		}

		let value_result = config_state_guard.get_value(
			section_key_opt.as_deref(),
			overrides.resource.as_ref(), // Pass resource for potential scope logic
		);

		debug!(
			"[Env CfgProv GetConfig] Value for section {:?}: (sample) {}...",
			section_key_opt,
			value_result.to_string().chars().take(70).collect::<String>()
		);
		Ok(value_result)
	}

	async fn update_configuration_value(
		&self,
		key_to_update:String,
		value_to_set:Value, // If Value::Null, effectively removes the key
		target_scope:ConfigurationTarget,
		overrides:IConfigurationOverrides, // For resource URI (if WORKSPACE_FOLDER) and languageId
		scope_to_language_override:Option<bool>,
	) -> Result<(), CommonError> {
		info!(
			"[Env CfgProv UpdateConfig] Request: key='{}', target_scope={:?}, value_is_null={}, scope_to_lang={:?}, \
			 override_resource={:?}",
			key_to_update,
			target_scope,
			value_to_set.is_null(),
			scope_to_language_override,
			overrides.resource.as_ref().and_then(|v| v.get("external"))
		);

		let app_state = self.get_app_state();

		// 1. Determine the target settings.json file path.
		let target_config_file_path = handlers::config::get_config_path_for_target(
			&self.app_handle,
			&app_state,
			target_scope,
			&overrides,
			scope_to_language_override.unwrap_or(false),
		)?;
		info!(
			"[Env CfgProv UpdateConfig] Target config file for update: {}",
			target_config_file_path.display()
		);

		// 2. Load the current content of that specific settings file.
		let mut current_target_file_json_content =
			handlers::config::load_json_file_if_exists_or_default(&target_config_file_path).await?;
		trace!(
			"[Env CfgProv UpdateConfig] Loaded JSON ({} top-level keys) from target file '{}'",
			current_target_file_json_content.as_object().map_or(0, |m| m.keys().len()),
			target_config_file_path.display()
		);

		// 3. Update the value at the specified key within the loaded JSON content.
		let mut effective_json_node_to_update_in = &mut current_target_file_json_content;
		let mut language_scope_key_holder:Option<String> = None; // To keep string alive for entry()

		if scope_to_language_override.unwrap_or(false) {
			if let Some(lang_id_str) = &overrides.override_identifier {
				language_scope_key_holder = Some(format!("[{}]", lang_id_str)); // e.g., "[typescript]"
				let lang_scope_key_ref = language_scope_key_holder.as_ref().unwrap();

				if !effective_json_node_to_update_in.is_object() {
					*effective_json_node_to_update_in = serde_json::json!({}); // Ensure it's an object
				}
				effective_json_node_to_update_in = effective_json_node_to_update_in
                    .as_object_mut().unwrap() // Safe due to check above
                    .entry(lang_scope_key_ref.clone())
                    .or_insert_with(|| serde_json::json!({}));
			} else {
				warn!(
					"[Env CfgProv UpdateConfig] 'scopeToLanguage' true for key '{}', but no languageId. Updating \
					 top-level of '{}'.",
					key_to_update,
					target_config_file_path.display()
				);
			}
		}
		handlers::config::update_json_value_at_path(effective_json_node_to_update_in, &key_to_update, value_to_set);
		trace!(
			"[Env CfgProv UpdateConfig] Key '{}' updated in-memory for file '{}'.",
			key_to_update,
			target_config_file_path.display()
		);

		// 4. Write the modified JSON content back to the target settings file.
		handlers::config::write_json_file(&target_config_file_path, current_target_file_json_content).await?;
		info!(
			"[Env CfgProv UpdateConfig] Successfully wrote updated config to file: {}",
			target_config_file_path.display()
		);

		// 5. Reload and re-merge all configurations into AppState.configuration.
		let new_merged_config_state =
			handlers::config::load_and_merge_configurations_internal(&self.app_handle, &app_state).await?;
		app_state
			.configuration
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.update_from_new_state(new_merged_config_state);
		info!(
			"[Env CfgProv UpdateConfig] In-memory AppState.configuration reloaded after change to file '{}'.",
			target_config_file_path.display()
		);

		// 6. Notify Cocoon (and other listeners) that configuration has changed.
		handlers::config::notify_config_changed_for_keys(&self.app_handle, vec![key_to_update]).await;
		Ok(())
	}
}

// --- ConfigInspector Implementation ---
#[async_trait]
impl ConfigInspector for MountainEnvironment {
	async fn inspect_configuration_value(
		&self,
		key:String,
		overrides:IConfigurationOverrides,
	) -> Result<Option<InspectResultData>, CommonError> {
		info!(
			"[Env CfgInsp] Inspecting config key='{}', overrides.resource={:?}",
			key,
			overrides.resource.as_ref().and_then(|v| v.get("external"))
		);

		// For MVP, this is partially stubbed. It returns the effective value but
		// doesn't populate all other scope-specific values by reading individual
		// files.
		let app_state = self.get_app_state();
		let config_guard = app_state
			.configuration
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		// Get effective value using the same logic as
		// ConfigProvider.get_configuration_value. This uses the merged configuration.
		let effective_value = config_guard.get_value(Some(&key), overrides.resource.as_ref());

		if effective_value.is_null() && !config_guard.data.get(&key).is_some() {
			// Key is not found even as null in the effective configuration.
			debug!(
				"[Env CfgInsp] Key '{}' not found in effective configuration. Returning None.",
				key
			);
			Ok(None)
		} else {
			// TODO: Implement full inspection logic:
			// 1. Load User settings.json, check for `key` (and lang-specific if
			//    applicable).
			// 2. Load Workspace .code-workspace file, check `settings` object for `key`.
			// 3. Determine relevant Workspace Folder, load its .vscode/settings.json, check
			//    for `key`.
			// 4. Load default values.
			// 5. Construct `InspectResultData` populating all relevant `..._value` fields.
			warn!(
				"[Env CfgInsp] inspect_configuration_value STUBBED for non-effective values. Returning effective \
				 value only for key '{}'.",
				key
			);
			Ok(Some(InspectResultData {
				effective_value:Some(effective_value),
				// Other fields are default (None)
				..Default::default()
			}))
		}
	}
}

// --- Requires Implementations ---
impl Requires<Arc<dyn ConfigProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}
