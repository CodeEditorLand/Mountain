//! Configuration value retrieval.
//!
//! Implements `GetConfigurationValue` for `MountainEnvironment`. Reads
//! from the pre-merged `ApplicationState::Configuration::GlobalConfiguration`
//! cache - no disk I/O on the hot path.
//!
//! If `section` is `None`, the entire merged object is returned. If it
//! is `Some("a.b.c")`, the key is split on `.` and the function walks
//! the nested JSON tree one segment at a time, returning `Value::Null`
//! (not an error) for any missing intermediate or leaf node. This
//! matches VS Code's behaviour where `getConfiguration('a.b').get('c')`
//! returns `undefined` rather than throwing.

use CommonLibrary::{
	Configuration::DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	Error::CommonError::CommonError,
};
use serde_json::Value;

use crate::dev_log;

/// Retrieves a configuration value from the cached, merged configuration.
/// When `overrides.OverrideIdentifier` is set, language-scoped values
/// from `[<language>]` blocks in settings.json take precedence over base
/// values.
pub(super) async fn get_configuration_value(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	section:Option<String>,

	overrides:ConfigurationOverridesDTO,
) -> Result<Value, CommonError> {
	dev_log!(
		"config",
		"[ConfigurationProvider] Getting configuration for section: {:?} (language: {:?})",
		section,
		overrides.OverrideIdentifier
	);

	let configuration_guard = environment
		.ApplicationState
		.Configuration
		.GlobalConfiguration
		.lock()
		.map_err(|e| CommonError::StateLockPoisoned { Context:format!("Failed to lock configuration: {}", e) })?;

	// Base value from merged config.
	let base_value = match section.as_deref() {
		None => (*configuration_guard).clone(),

		Some(section_path) => {
			let mut current = &*configuration_guard;

			for key in section_path.split('.') {
				current = match current.get(key) {
					Some(value) => value,

					None => {
						dev_log!(
							"config",
							"warn: [ConfigurationProvider] Configuration section '{}' not found in path: {:?}",
							key,
							section_path
						);

						return Ok(Value::Null);
					},
				};
			}

			current.clone()
		},
	};

	// If a language override is requested, check for `[<language>]` blocks in
	// the merged config and overlay any matching keys on top of the base value.
	// VS Code uses `[rust]`, `[typescript]`, etc. as top-level keys.
	let configuration_value = if let Some(ref lang_id) = overrides.OverrideIdentifier {
		let lang = lang_id.as_str();
		let lang_block_key = format!("[{}]", lang);
		if let Some(lang_block) = configuration_guard.get(&lang_block_key).and_then(|v| v.as_object()) {
			match section.as_deref() {
				None => {
					// Return the whole merged config with language block applied.
					let mut merged = if let Some(obj) = base_value.as_object() {
						obj.clone()
					} else {
						return Ok(base_value);
					};
					for (k, v) in lang_block {
						merged.insert(k.clone(), v.clone());
					}
					Value::Object(merged)
				},
				Some(section_path) => {
					// Check if the language block overrides this specific section key.
					let top_key = section_path.split('.').next().unwrap_or(section_path);
					if let Some(lang_value) = lang_block.get(top_key) {
						let remainder:Vec<&str> = section_path.splitn(2, '.').skip(1).collect();
						if remainder.is_empty() {
							lang_value.clone()
						} else {
							let mut cur = lang_value;
							for k in remainder[0].split('.') {
								match cur.get(k) {
									Some(v) => cur = v,
									None => return Ok(base_value),
								}
							}
							cur.clone()
						}
					} else {
						base_value
					}
				},
			}
		} else {
			base_value
		}
	} else {
		base_value
	};

	// Validate that the configuration value exists
	if configuration_value.is_null() {
		dev_log!(
			"config",
			"warn: [ConfigurationProvider] Configuration section not found: {:?}",
			section
		);
	}

	Ok(configuration_value)
}
