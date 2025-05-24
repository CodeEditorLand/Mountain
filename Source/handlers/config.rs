// ---------------------------------------------------------------------------------------------
// Mountain Configuration Handlers & Helpers (handlers/config.rs)
// --------------------------------------------------------------------------------------------
// Provides helper functions for managing configuration settings, including path
// resolution, file I/O for settings.json, JSON manipulation, merging, and
// notifying sidecars of changes. These functions are primarily used by the
// ConfigProvider and ConfigInspector implementations in `environment.rs`.
//
// Responsibilities:
// - Resolving the path to the correct settings.json file based on target scope
//   (User, Workspace, WorkspaceFolder) and resource overrides.
// - Asynchronously loading configuration data from a JSON file.
// - Asynchronously writing configuration data to a JSON file, creating
//   directories if necessary.
// - Modifying nested JSON structures to set or remove configuration values at a
//   given key path.
// - Performing a simplified merge of user and workspace/folder configurations.
// - Sending notifications to Cocoon when configuration changes.
// - Providing a document selector matching utility with glob support via
//   `globset`, noting limitations for complex relative/base-URI patterns.
//
// Key Interactions:
// - Called by `environment.rs` (ConfigProvider/ConfigInspector
//   implementations).
// - Interacts with `AppState` to get workspace folder information and config
//   paths.
// - Uses `tokio::fs` for asynchronous file operations.
// - Uses `serde_json` for JSON processing.
// - Uses `vine` to send notifications to sidecars.
// --------------------------------------------------------------------------------------------

use std::{
	path::{Path, PathBuf},

	// Kept from original, though direct use here might be minimal for pure helpers
	sync::Arc,
};

use Land_Common::{
	config_effects::{ConfigurationTarget, IConfigurationOverrides},
	errors::CommonError,
};
// For document selector matching
use globset::{Error as GlobsetError, Glob, GlobBuilder, GlobMatcher};
use log::{debug, error, info, trace, warn};
// For DocumentFilterDto
use serde::Deserialize;
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime};
// Explicitly use tokio::fs
use tokio::fs;
use url::Url;

use crate::{
	// For updating AppState.configuration
	app_state::{AppState, ConfigurationState},

	vine,
};

// --- Helper: Error Creation & Mapping ---
fn config_error(operation:&str, message:String) -> CommonError {
	error!("[ConfigHelpers] Error during {}: {}", operation, message);

	match operation {
		"path_resolution" => CommonError::ConfigUpdate("path_resolution".to_string(), message),

		"load" => CommonError::ConfigLoad(message),

		"write" | "serialize" | "create_dir" | "create_dir_check" => {
			CommonError::ConfigUpdate(operation.to_string(), message)
		},

		_ => CommonError::Unknown(format!("Config operation '{}' failed: {}", operation, message)),
	}
}

// Map lock errors for AppState access
fn map_lock_error<T>(e:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> CommonError {
	CommonError::StateLock(format!("Config helper AppState lock error: {}", e))
}

// --- Public Helper Functions ---

/// Resolves the absolute path to the `settings.json` file for a given target
/// and scope. `_scope_to_language` is present for API consistency but not used
/// in MVP path resolution.
pub fn get_config_path_for_target<R:TauriRuntime>(
	app_handle:&AppHandle<R>,

	app_state:&AppState,

	target:ConfigurationTarget,

	overrides:&IConfigurationOverrides,

	_scope_to_language:bool,
) -> Result<PathBuf, CommonError> {
	trace!(
		"[ConfigHelpers] Resolving config path: target={:?}, overrides.resource={:?}, overrides.langId={:?}",
		target,
		overrides.resource.as_ref().and_then(|v| v.get("external")),
		overrides.override_identifier
	);

	let path_resolver = app_handle.path_resolver();

	let base_user_config_dir = path_resolver.app_config_dir().ok_or_else(|| {
		config_error(
			"path_resolution",
			"Cannot resolve app config directory (for User settings)".to_string(),
		)
	})?;

	match target {
		ConfigurationTarget::USER_LOCAL | ConfigurationTarget::USER => {
			let user_settings_path = base_user_config_dir.join("User").join("settings.json");

			debug!("[ConfigHelpers] Resolved User settings path: {}", user_settings_path.display());

			Ok(user_settings_path)
		},

		ConfigurationTarget::WORKSPACE => {
			let config_path_guard = app_state.workspace_config_path.lock().map_err(map_lock_error)?;

			let ws_path = config_path_guard.as_ref().cloned().ok_or_else(|| {
				config_error(
					"path_resolution",
					"No workspace configuration file (.code-workspace) loaded. Cannot target WORKSPACE settings."
						.to_string(),
				)
			})?;

			debug!(
				"[ConfigHelpers] Resolved Workspace settings path (from .code-workspace): {}",
				ws_path.display()
			);

			Ok(ws_path)
		},

		ConfigurationTarget::WORKSPACE_FOLDER => {
			let resource_uri_val = overrides.resource.as_ref().ok_or_else(|| {
				config_error(
					"path_resolution",
					"Missing resource URI for WORKSPACE_FOLDER target".to_string(),
				)
			})?;

			let resource_uri_str = resource_uri_val
				.get("external")
				.and_then(Value::as_str)
				.or_else(|| {
					resource_uri_val.get("path").and_then(Value::as_str).map(|p_str| {
						if Path::new(p_str).is_absolute() {
							Url::from_file_path(p_str)
								.map(|u| u.to_string())
								.unwrap_or_else(|_| p_str.to_string())
						} else {
							p_str.to_string()
						}
					})
				})
				.ok_or_else(|| {
					CommonError::InvalidArg(
						"resource".to_string(),
						"Resource override is not a valid URI string or component with 'external' or 'path'"
							.to_string(),
					)
				})?;

			let resource_uri = Url::parse(resource_uri_str).map_err(|_| {
				CommonError::InvalidArg(
					"resource".to_string(),
					format!("Invalid resource URI in overrides: {}", resource_uri_str),
				)
			})?;

			let folders_guard = app_state.workspace_folders.lock().map_err(map_lock_error)?;

			let containing_folder = folders_guard
				.iter()
				.find(|f| resource_uri.scheme() == f.uri.scheme() && resource_uri.path().starts_with(f.uri.path()))
				.ok_or_else(|| {
					config_error(
						"path_resolution",
						format!("Resource URI '{}' does not belong to any known workspace folder.", resource_uri),
					)
				})?;

			if containing_folder.uri.scheme() != "file" {
				return Err(config_error(
					"path_resolution",
					format!(
						"Cannot get folder settings for non-file scheme folder: {}",
						containing_folder.uri
					),
				));
			}

			let folder_settings_path = PathBuf::from(containing_folder.uri.path())
				.join(".vscode")
				.join("settings.json");

			debug!(
				"[ConfigHelpers] Resolved Workspace Folder settings path: {}",
				folder_settings_path.display()
			);

			Ok(folder_settings_path)
		},

		_ => {
			warn!(
				"[ConfigHelpers] ConfigurationTarget {:?} path resolution not implemented for general settings update.",
				target
			);

			Err(CommonError::NotImplemented(format!(
				"ConfigurationTarget {:?} path resolution not implemented.",
				target
			)))
		},
	}
}

/// Asynchronously loads JSON content from a file. Returns an empty JSON object
/// if not found or if file is empty.
pub async fn load_json_file_if_exists_or_default(path:&Path) -> Result<Value, CommonError> {
	trace!("[ConfigHelpers Io] Attempting to load JSON from: {}", path.display());

	match fs::read_to_string(path).await {
		Ok(content) => {
			if content.trim().is_empty() {
				debug!(
					"[ConfigHelpers Io] File is empty, returning default empty JSON object: {}",
					path.display()
				);

				Ok(json!({}))
			} else {
				serde_json::from_str(&content)
					.map_err(|e| config_error("load", format!("JSON parse failed for {}: {}", path.display(), e)))
			}
		},

		Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
			debug!(
				"[ConfigHelpers Io] File not found, returning default empty JSON object: {}",
				path.display()
			);

			Ok(json!({}))
		},

		Err(e) => Err(config_error("load", format!("File read failed for {}: {}", path.display(), e))),
	}
}

/// Asynchronously writes a `serde_json::Value` to a file, pretty-printed.
/// Creates parent directories if they don't exist.
pub async fn write_json_file(path:&Path, value:&Value) -> Result<(), CommonError> {
	trace!("[ConfigHelpers Io] Attempting to write JSON to: {}", path.display());

	let parent = path
		.parent()
		.ok_or_else(|| config_error("write", format!("Invalid path (has no parent): {}", path.display())))?;

	if !fs::try_exists(parent).await.map_err(|e| {
		config_error(
			"create_dir_check",
			format!("Failed to check existence of {}: {}", parent.display(), e),
		)
	})? {
		debug!(
			"[ConfigHelpers Io] Creating parent directory for config file: {}",
			parent.display()
		);

		fs::create_dir_all(parent).await.map_err(|e| {
			config_error("create_dir", format!("Failed to create directory {}: {}", parent.display(), e))
		})?;
	}

	let content = serde_json::to_string_pretty(value)
		.map_err(|e| config_error("serialize", format!("Failed to serialize JSON for {}: {}", path.display(), e)))?;

	let mut file = fs::File::create(path)
		.await
		.map_err(|e| config_error("write", format!("Failed to create/open for writing {}: {}", path.display(), e)))?;

	file.write_all(content.as_bytes())
		.await
		.map_err(|e| config_error("write", format!("Failed to write to {}: {}", path.display(), e)))?;

	info!("[ConfigHelpers Io] Successfully wrote JSON to {}", path.display());

	Ok(())
}

/// Updates a value in a `serde_json::Value` (assumed to be an Object) at a
/// given dot-separated key path. If `value_to_set` is `Value::Null`, the key is
/// removed. Intermediate objects are created if they don't exist.
pub fn update_json_value_at_path(target_value:&mut Value, key_path:&str, value_to_set:Value) {
	trace!(
		"[ConfigHelpers Json] Updating value at path: '{}', new value (is_null): {}",
		key_path,
		value_to_set.is_null()
	);

	let mut current = target_value;

	let parts:Vec<&str> = key_path.split('.').collect();

	if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
		warn!(
			"[ConfigHelpers Json] Attempted to update with empty or invalid key path: '{}'. Doing nothing.",
			key_path
		);

		if key_path.is_empty() && target_value.is_object() && !value_to_set.is_null() && value_to_set.is_object() {
			debug!("[ConfigHelpers Json] Empty key path, replacing entire object.");

			*current = value_to_set;
		}

		return;
	}

	let last_part_index = parts.len() - 1;

	for (i, part_str) in parts.iter().enumerate() {
		let part_key = part_str.to_string();

		if part_key.is_empty() {
			warn!("[ConfigHelpers Json] Empty segment in key path '{}', skipping.", key_path);

			continue;
		}

		if i == last_part_index {
			if let Some(obj) = current.as_object_mut() {
				if value_to_set.is_null() {
					trace!("[ConfigHelpers Json] Removing key: '{}'", part_key);

					obj.remove(&part_key);
				} else {
					trace!("[ConfigHelpers Json] Setting key: '{}'", part_key);

					obj.insert(part_key, value_to_set);
				}
			} else {
				warn!(
					"[ConfigHelpers Json] Cannot set key '{}' in path '{}' because parent is not an object.",
					part_key, key_path
				);
			}

			return;
		} else {
			if !current.is_object() {
				debug!(
					"[ConfigHelpers Json] Path segment '{}' in '{}' is not an object, creating new object node.",
					part_key, key_path
				);

				*current = json!({});
			}

			current = current.as_object_mut().unwrap().entry(part_key).or_insert_with(|| json!({}));
		}
	}
}

/// Loads configurations from user, workspace, and folder settings,
/// and performs a simplified merge for MVP.
pub async fn load_and_merge_configurations_internal<R:TauriRuntime>(
	app_handle:&AppHandle<R>,

	app_state:&AppState,
) -> Result<ConfigurationState, CommonError> {
	info!("[ConfigHelpers Merge] Loading and merging all configurations...");

	let user_config_path = get_config_path_for_target(
		app_handle,
		app_state,
		ConfigurationTarget::USER_LOCAL,
		&IConfigurationOverrides::default(),
		false,
	)?;

	let mut merged_config_data = load_json_file_if_exists_or_default(&user_config_path).await?;

	debug!(
		"[ConfigHelpers Merge] Loaded user config ({} top-level keys) from: {}",
		merged_config_data.as_object().map_or(0, |m| m.len()),
		user_config_path.display()
	);

	let workspace_config_path_opt = app_state.workspace_config_path.lock().map_err(map_lock_error)?.clone();

	if let Some(ws_config_path) = workspace_config_path_opt {
		if fs::try_exists(&ws_config_path).await.unwrap_or(false) && ws_config_path.is_file() {
			let workspace_file_values = load_json_file_if_exists_or_default(&ws_config_path).await?;

			if let Some(settings_in_workspace_file) = workspace_file_values.get("settings").cloned() {
				if settings_in_workspace_file.is_object() {
					debug!(
						"[ConfigHelpers Merge] Loaded workspace settings ({} top-level keys) from .code-workspace: {}",
						settings_in_workspace_file.as_object().map_or(0, |m| m.len()),
						ws_config_path.display()
					);

					merge_json_values(&mut merged_config_data, &settings_in_workspace_file);
				} else {
					warn!(
						"[ConfigHelpers Merge] 'settings' key in {} is not an object.",
						ws_config_path.display()
					);
				}
			} else {
				trace!(
					"[ConfigHelpers Merge] No 'settings' key in workspace config file: {}",
					ws_config_path.display()
				);
			}
		} else {
			warn!(
				"[ConfigHelpers Merge] Workspace config path {} does not exist or is not a file.",
				ws_config_path.display()
			);
		}
	} else {
		trace!("[ConfigHelpers Merge] No .code-workspace file configured.");
	}

	let folders_guard = app_state.workspace_folders.lock().map_err(map_lock_error)?;

	for folder_state in folders_guard.iter() {
		if folder_state.uri.scheme() == "file" {
			let folder_path = PathBuf::from(folder_state.uri.path());

			let folder_settings_path = folder_path.join(".vscode").join("settings.json");

			if fs::try_exists(&folder_settings_path).await.unwrap_or(false) {
				let folder_values = load_json_file_if_exists_or_default(&folder_settings_path).await?;

				debug!(
					"[ConfigHelpers Merge] Loaded folder settings ({} top-level keys) for '{}' from: {}",
					folder_values.as_object().map_or(0, |m| m.len()),
					folder_state.name,
					folder_settings_path.display()
				);

				merge_json_values(&mut merged_config_data, &folder_values);
			} else {
				trace!(
					"[ConfigHelpers Merge] No settings.json found for folder '{}': {}",
					folder_state.name,
					folder_settings_path.display()
				);
			}
		} else {
			warn!(
				"[ConfigHelpers Merge] Skipping folder settings for non-file scheme folder: {}",
				folder_state.uri
			);
		}
	}

	drop(folders_guard);

	info!(
		"[ConfigHelpers Merge] Configuration merge complete. Effective top-level keys: {}",
		merged_config_data.as_object().map_or(0, |m| m.len())
	);

	trace!(
		"[ConfigHelpers Merge] Final merged data (sample): {}",
		merged_config_data.to_string().chars().take(200).collect::<String>()
	);

	Ok(ConfigurationState::new(merged_config_data))
}

/// Simple recursive merge for JSON values. `source` overrides `target`.
/// Arrays in source will replace arrays in target. Nulls in source will replace
/// values in target.
fn merge_json_values(target:&mut Value, source:&Value) {
	match (target, source) {
		(Value::Object(target_map), Value::Object(source_map)) => {
			for (key, source_val) in source_map {
				let target_entry = target_map.entry(key.clone()).or_insert_with(|| source_val.clone());

				if target_entry.is_object() && source_val.is_object() {
					merge_json_values(target_entry, source_val);
				} else {
					*target_entry = source_val.clone();
				}
			}
		},

		(target_val, source_val) => {
			*target_val = source_val.clone();
		},
	}
}

/// Notifies Cocoon (via Vine) that specified configuration keys have changed.
pub async fn notify_config_changed_for_keys<R:TauriRuntime>(app_handle:&AppHandle<R>, affected_keys:Vec<String>) {
	if affected_keys.is_empty() {
		trace!("[ConfigHelpers Notify] notify_config_changed_for_keys called with no keys. Skipping.");

		return;
	}

	info!(
		"[ConfigHelpers Notify] Notifying Cocoon of config change. Affected keys: {:?}",
		affected_keys
	);

	let app_state = app_handle.state::<AppState>();

	let (config_init_data_dto, change_event_dto) = {
		let config_guard = app_state
			.configuration
			.lock()
			.map_err(map_lock_error)
			.expect("Config lock failed for notification");

		let effective_config = config_guard.data.clone();

		let config_init_data_dto = json!({

			"effective": effective_config,

			"defaults": { "contents": {} },

			"user": { "contents": {} },

			"workspace": { "contents": {} },

			"folders": [],

			"memory": { "contents": {} },

			"policy": Value::Null,

			"configurationScopes": []
		});

		let change_event_dto = json!({ "keys": affected_keys, "overrides": [] });

		(config_init_data_dto, change_event_dto)
	};

	let payload_to_send = json!([config_init_data_dto, change_event_dto]);

	trace!(
		"[ConfigHelpers Notify] $acceptConfigurationChanged payload (brief): keys: {:?}, effective keys: {}",
		affected_keys,
		payload_to_send[0]["effective"].as_object().map_or(0, |o| o.keys().len())
	);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptConfigurationChanged".to_string(), payload_to_send)
			.await
	{
		error!(
			"[ConfigHelpers Notify] Failed to send $acceptConfigurationChanged notification: {}",
			e
		);
	} else {
		debug!(
			"[ConfigHelpers Notify] $acceptConfigurationChanged notification sent successfully for keys: {:?}",
			affected_keys
		);
	}
}

// DocumentFilter DTO
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFilterDto {
	pub language:Option<String>,

	pub scheme:Option<String>,

	pub pattern:Option<String>,

	// UriComponents expected
	pub base_uri:Option<Value>,
}

/// Document Selector matching utility using `globset`.
///
/// `selector_val`: A `serde_json::Value` representing a `DocumentSelector`.
/// `document_uri`: The URI of the document to match.
/// `language_id`: The language ID of the document.
///
/// **Current Limitations & TODOs for Glob Patterns:**
/// - **Base URI Handling:** If `filter.base_uri` is present in a
///   `DocumentFilterDto`, the `pattern` should ideally be resolved relative to
///   this base URI before matching against a path derived from `document_uri`.
///   This is not yet implemented.
/// - **Workspace-Relative Globs:** If a `pattern` is relative (e.g.,
///
///   "src/**/*.ts") and no `base_uri` is specified, VS Code often attempts to
///   match it against paths relative to each workspace folder root. This
///   advanced multi-root relative matching is not yet implemented.
/// - **Current Behavior:** This function primarily matches glob patterns
///   against the `document_uri.path()` (the full, percent-decoded path). This
///   works well for:
///     - Absolute path globs (e.g., `/project/src/**/*.ts`).
///     - Globs that match the end of a path (e.g., `**/*.ts`, `*.txt` if
///       written as `**/*.txt`).
pub fn match_document_selector(selector_val:&Value, document_uri:&Url, language_id:&str) -> bool {
	trace!(
		"[SelectorMatch] Selector: {:?}, Doc URI: '{}', Lang: '{}'",
		selector_val.as_str().unwrap_or("<object/array>"),
		document_uri.as_str(),
		language_id
	);

	if let Some(selector_lang_id_str) = selector_val.as_str() {
		let matches = selector_lang_id_str == language_id || selector_lang_id_str == "*";

		trace!(
			"[SelectorMatch] String selector '{}' vs lang '{}': {}",
			selector_lang_id_str, language_id, matches
		);

		return matches;
	}

	if let Some(selector_array) = selector_val.as_array() {
		// Any one filter in the array must match
		let matches = selector_array
			.iter()
			.any(|s_filter| match_document_selector(s_filter, document_uri, language_id));

		trace!("[SelectorMatch] Array selector result: {}", matches);

		return matches;
	}

	if selector_val.is_object() {
		match serde_json::from_value::<DocumentFilterDto>(selector_val.clone()) {
			Ok(filter) => {
				trace!("[SelectorMatch] Parsed DocumentFilterDto: {:?}", filter);

				// 1. Language Check
				if let Some(filter_lang) = &filter.language {
					if !(filter_lang == language_id || filter_lang == "*") {
						trace!(
							"[SelectorMatch] Language mismatch: filter='{}', doc='{}'",
							filter_lang, language_id
						);

						return false;
					}
				}

				trace!("[SelectorMatch] Language match success (or not specified).");

				// 2. Scheme Check
				if let Some(filter_scheme) = &filter.scheme {
					if !(filter_scheme == document_uri.scheme() || filter_scheme == "*") {
						trace!(
							"[SelectorMatch] Scheme mismatch: filter='{}', doc='{}'",
							filter_scheme,
							document_uri.scheme()
						);

						return false;
					}
				}

				trace!("[SelectorMatch] Scheme match success (or not specified).");

				// 3. Pattern Check (Glob)
				if let Some(filter_pattern_str) = &filter.pattern {
					if filter.base_uri.is_some() {
						warn!(
							"[SelectorMatch] Glob pattern '{}' has a 'base_uri' specified, but base URI resolution \
							 for globs is NOT YET IMPLEMENTED. Pattern will be matched against the full document path.",
							filter_pattern_str
						);
					}

					let path_to_match = document_uri.path();

					trace!(
						"[SelectorMatch] Glob: pattern='{}', attempting match against path='{}'",
						filter_pattern_str, path_to_match
					);

					let mut glob_builder = GlobBuilder::new(filter_pattern_str);

					if cfg!(windows) {
						glob_builder.case_insensitive(true);
					}

					glob_builder.literal_separator(false);

					match glob_builder.build() {
						Ok(glob) => {
							let matcher = glob.compile_matcher();

							if !matcher.is_match(path_to_match) {
								trace!(
									"[SelectorMatch] Glob pattern '{}' DID NOT MATCH path '{}'",
									filter_pattern_str, path_to_match
								);

								return false;
							}

							trace!(
								"[SelectorMatch] Glob pattern '{}' MATCHED path '{}'",
								filter_pattern_str, path_to_match
							);
						},

						Err(e) => {
							error!(
								"[SelectorMatch] Invalid glob pattern syntax '{}': {}. Treating as non-match.",
								filter_pattern_str, e
							);

							return false;
						},
					}
				}

				trace!("[SelectorMatch] Pattern match success (or not specified).");

				info!(
					"[SelectorMatch] Filter object MATCHED ALL CRITERIA for doc '{}', lang '{}': {:?}",
					document_uri.as_str(),
					language_id,
					filter
				);

				return true;
			},

			Err(e) => {
				warn!(
					"[SelectorMatch] Failed to deserialize selector object into DocumentFilterDto: {:?}. Error: {}. \
					 Treating as non-match.",
					selector_val, e
				);

				return false;
			},
		}
	}

	warn!(
		"[SelectorMatch] Unrecognized selector format (not string, array, or filter object), no match: {:?}",
		selector_val
	);

	false
}
