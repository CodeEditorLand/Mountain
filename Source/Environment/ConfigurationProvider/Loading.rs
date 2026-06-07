//! Configuration loading, caching, and merging.
//!
//! Provides the three public entry points consumed by the rest of the
//! `ConfigurationProvider` module:
//!
//! - `read_and_parse_configuration_file` - reads a single `settings.json` from
//!   disk via the async `ApplicationRunTime`, with a 250 ms TTL parse cache to
//!   avoid redundant disk reads during burst `Inspect` calls.
//! - `initialize_and_merge_configurations` - rebuilds the merged
//!   `GlobalConfiguration` by layering Default → User → Workspace in precedence
//!   order (deep-merge for nested objects, shallow for root keys).
//! - `collect_default_configurations` - walks every scanned extension's
//!   `contributes.configuration.properties` map and extracts `default` values,
//!   inserting them into a nested map keyed by dotted path.
//! - `ClearSettingsFileCache` - invalidates the parse cache; called by
//!   `UpdateValue` after any write so the next read sees fresh content.

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Arc, Mutex, OnceLock},
	time::{Duration, Instant},
};

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::ReadFile::ReadFile,
};

use serde_json::{Map, Value};

use tauri::Manager;

use crate::{
	ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO,
	Environment::Utility,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Short TTL cache for parsed `settings.json` reads. The
/// `InspectConfigurationValue` handler reads BOTH the user
/// settings.json and the workspace settings.json on every call;
/// log audit `20260501T053137` shows ~57 Inspect calls per session
/// = 114 disk reads of the same one or two files. With this cache,
/// repeated reads within `TTL_MS` reuse the parsed `Value` and a
/// burst of Inspects collapses to ~1 disk read per file. TTL is
/// short enough (250ms) that user edits to settings.json show up
/// within a quarter-second.
const SETTINGS_FILE_CACHE_TTL_MS:u64 = 250;

struct CachedSettingsValue {

	StoredAt:Instant,

	Parsed:Value,
}

fn SettingsFileCache() -> &'static Mutex<HashMap<PathBuf, CachedSettingsValue>> {

	static CACHE:OnceLock<Mutex<HashMap<PathBuf, CachedSettingsValue>>> = OnceLock::new();

	CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Drop every cached settings.json parse. Caller: any code path
/// that mutates settings (`UpdateConfigurationValue`,
/// `initialize_and_merge_configurations`).
pub(crate) fn ClearSettingsFileCache() {

	if let Ok(mut Guard) = SettingsFileCache().lock() {
		Guard.clear();
	}
}

/// An internal helper to read and parse a single JSON configuration file.
pub(super) async fn read_and_parse_configuration_file(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	path:&Option<PathBuf>,
) -> Result<Value, CommonError> {

	if let Some(p) = path {
		// Cache check: return a clone of the parsed value if the same
		// file was read within the TTL window.
		if let Ok(Guard) = SettingsFileCache().lock() {
			if let Some(Entry) = Guard.get(p) {
				if Entry.StoredAt.elapsed() < Duration::from_millis(SETTINGS_FILE_CACHE_TTL_MS) {
					return Ok(Entry.Parsed.clone());
				}
			}
		}

		let runtime = environment.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		if let Ok(bytes) = runtime.Run(ReadFile(p.clone())).await {
			let Parsed = serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::Object(Map::new()));

			if let Ok(mut Guard) = SettingsFileCache().lock() {
				Guard.insert(
					p.clone(),

					CachedSettingsValue { StoredAt:Instant::now(), Parsed:Parsed.clone() },
				);
			}

			return Ok(Parsed);
		}
	}

	Ok(Value::Object(Map::new()))
}

/// Logic to load and merge all configuration files into the effective
/// configuration stored in `ApplicationState`.
pub async fn Fn(environment:&crate::Environment::MountainEnvironment::MountainEnvironment) -> Result<(), CommonError> {

	dev_log!(
		"config",

		"[ConfigurationProvider] Re-initializing and merging all configurations..."
	);

	let default_config = collect_default_configurations(&environment.ApplicationState)?;

	let user_settings_path = environment
		.ApplicationHandle
		.path()
		.app_config_dir()
		.map(|p| p.join("settings.json"))
		.ok();

	let workspace_settings_path = environment
		.ApplicationState
		.Workspace
		.WorkspaceConfigurationPath
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
		.clone();

	let user_config = read_and_parse_configuration_file(environment, &user_settings_path).await?;

	let workspace_config = read_and_parse_configuration_file(environment, &workspace_settings_path).await?;

	// A true deep merge is required here. The merge order matches the cascade:
	// Default (base) → User (overrides default) → Workspace (overrides user)
	let mut merged = default_config.as_object().cloned().unwrap_or_default();

	if let Some(user_map) = user_config.as_object() {
		for (key, value) in user_map {
			// Deep merge nested objects, shallow merge at root level
			if value.is_object() && merged.get(key.as_str()).is_some_and(|v| v.is_object()) {
				if let (Some(user_value), Some(_base_value)) =
					(value.as_object(), merged.get(key.as_str()).and_then(|v| v.as_object()))

				{
					for (inner_key, inner_value) in user_value {
						merged.get_mut(key.as_str()).and_then(|v| v.as_object_mut()).map(|m| {
							m.insert(inner_key.clone(), inner_value.clone());
						});
					}
				}
			} else {
				merged.insert(key.clone(), value.clone());
			}
		}
	}

	if let Some(workspace_map) = workspace_config.as_object() {
		for (key, value) in workspace_map {
			if value.is_object() && merged.get(key.as_str()).is_some_and(|v| v.is_object()) {
				if let (Some(workspace_value), Some(_base_value)) =
					(value.as_object(), merged.get(key.as_str()).and_then(|v| v.as_object()))

				{
					for (inner_key, inner_value) in workspace_value {
						merged.get_mut(key.as_str()).and_then(|v| v.as_object_mut()).map(|m| {
							m.insert(inner_key.clone(), inner_value.clone());
						});
					}
				}
			} else {
				merged.insert(key.clone(), value.clone());
			}
		}
	}

	let configuration_size = merged.len();

	let final_config = MergedConfigurationStateDTO::Create(Value::Object(merged));

	*environment
		.ApplicationState
		.Configuration
		.GlobalConfiguration
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)? = final_config.Data;

	dev_log!(
		"config",

		"[ConfigurationProvider] Configuration merged successfully with {} top-level keys.",

		configuration_size
	);

	Ok(())
}

/// Collects default configurations from all installed extensions.
///
/// Reads each extension's `contributes.configuration` entry and pulls
/// the `default` value out of every property declaration. Stock VS Code
/// extensions (vscode.git, vscode.npm, gitlens, etc.) declare their
/// settings via the `properties` map shape:
///
/// ```jsonc
/// "contributes": {
///   "configuration": {
///     "title": "Git",
///     "properties": {
///       "git.enabled":                 { "type": "boolean", "default": true,  "description": "…" },
///       "git.path":                    { "type": ["string","array"], "default": null, "description": "…" },
///       "git.autoRepositoryDetection": { "type": ["boolean","string"], "default": true, "description": "…" }
///     }
///   }
/// }
/// ```
///
/// The previous implementation searched for a `[ {key, value} ]` array
/// shape that doesn't exist in any real VS Code manifest, so EVERY
/// `vscode.workspace.getConfiguration(...).get('foo')` lookup fell
/// through to undefined. Extensions that use the lookup's first arg
/// alone (no explicit default) saw undefined and silently bailed -
/// which is the failure mode behind vscode.git activating but never
/// reaching `vscode.scm.createSourceControl(...)`.
///
/// `contributes.configuration` accepts BOTH a single object AND an
/// array of objects (older multi-section schema), so we walk both
/// shapes and recursively dive into `properties`. The dotted key
/// (`git.enabled`) is split into a nested map shape so callers using
/// `inspect_configuration_value`'s `path.split('.').try_fold(...)`
/// land on the right node.
pub(super) fn collect_default_configurations(
	application_state:&crate::ApplicationState::State::ApplicationState::ApplicationState,
) -> Result<Value, CommonError> {

	let mut default_config = Map::new();

	for extension in application_state
		.Extension
		.ScannedExtensions
		.ScannedExtensions
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
		.values()

	{
		let Some(contributes) = &extension.Contributes else {
			continue;
		};

		let Some(configuration) = contributes.get("configuration") else {
			continue;
		};

		// Walk EITHER an array of {properties} blocks OR a single one.
		let blocks:Vec<&Value> = if let Some(array) = configuration.as_array() {
			array.iter().collect()
		} else {
			vec![configuration]
		};

		for block in blocks {
			let Some(properties) = block.get("properties").and_then(|p| p.as_object()) else {
				continue;
			};

			for (DottedKey, schema) in properties {
				let Some(default) = schema.get("default") else {
					continue;
				};

				InsertDottedDefault(&mut default_config, DottedKey, default.clone());
			}
		}
	}

	Ok(Value::Object(default_config))
}

/// Insert a value into `target` at the dotted path `git.enabled`,
/// creating intermediate object nodes as needed. Mirrors
/// `inspect_configuration_value`'s `try_fold` traversal so a lookup
/// for `git.enabled` finds `target["git"]["enabled"]`.
fn InsertDottedDefault(target:&mut Map<String, Value>, dotted:&str, value:Value) {

	let parts:Vec<&str> = dotted.split('.').collect();

	if parts.is_empty() {
		return;
	}

	if parts.len() == 1 {
		target.insert(parts[0].to_string(), value);

		return;
	}

	let head = parts[0];

	let entry = target.entry(head.to_string()).or_insert_with(|| Value::Object(Map::new()));

	if !entry.is_object() {
		// Conflicting prior insert (e.g. another extension declared
		// `git` as a non-object). Replace with a fresh map so we don't
		// silently drop the deeper key. Last-writer-wins matches the
		// merge precedence in `initialize_and_merge_configurations`.
		*entry = Value::Object(Map::new());
	}

	if let Some(child) = entry.as_object_mut() {
		// Walk the rest of the dotted path recursively. Re-build a
		// `Map<String, Value>` and insert from there, then move it
		// back. (Borrow-checker-friendly variant of in-place
		// recursion.)
		let mut sub = std::mem::take(child);

		let RemainingDotted = parts[1..].join(".");

		InsertDottedDefault(&mut sub, &RemainingDotted, value);

		*child = sub;
	}
}
