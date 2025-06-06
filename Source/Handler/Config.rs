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

use std::path::{Path, PathBuf};

use Land_Common::{
	config_effects::{ConfigurationTarget, IConfigurationOverrides},
	errors::CommonError,
};
// For document selector matching
use globset::GlobBuilder;
use log::{debug, error, info, trace, warn};
// For DocumentFilterDto
use serde::Deserialize;
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime};
// Explicitly use tokio::fs and add tokio::io::AsyncWriteExt
use tokio::{fs, io::AsyncWriteExt};
use url::Url;

use crate::{
	// For updating AppState.configuration
	app_state::{AppState, ConfigurationState},

	vine,
};

// --- Helper: Error Creation & Mapping ---

/// Creates a `CommonError` specific to configuration operations.
///
/// # Argument
/// * `operation` - A string slice describing the config operation (e.g.,
///
///
///   "path_resolution", "load").
/// * `message` - The specific error message.
///
/// # Returns
/// A `CommonError` instance.
fn create_config_error(operation:&str, message:String) -> CommonError {
	// Log the error when it's created for internal diagnostics
	error!("[ConfigHelpers Error] During '{}': {}", operation, message);

	match operation {
		"path_resolution" => CommonError::ConfigUpdate("path_resolution".to_string(), message),

		"load" => CommonError::ConfigLoad(message),

		"write" | "serialize" | "create_dir" | "create_dir_check" => {
			CommonError::ConfigUpdate(operation.to_string(), message)
		},

		_ => CommonError::Unknown(format!("Config operation '{}' failed: {}", operation, message)),
	}
}

/// Maps a `PoisonError` from `AppState` Mutex locks to a `CommonError`.
///
/// # Argument
/// * `e` - The `PoisonError`.
///
/// # Returns
/// A `CommonError::StateLock`.
fn map_app_state_lock_error_to_common_error<T>(e:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> CommonError {
	let err_msg = format!("Config helper AppState lock error: {}", e);

	// Log the specific lock error
	error!("[ConfigHelpers LockErr] {}", err_msg);

	CommonError::StateLock(err_msg)
}

// --- Public Helper Functions ---

/// Resolves the absolute path to the `settings.json` file for a given target
/// and optional resource override.
///
/// The `_scope_to_language` parameter is currently unused in this MVP
/// implementation but is present for API consistency with VS Code's concepts.
///
/// # Argument
/// * `app_handle` - Tauri `AppHandle` for path resolution.
/// * `app_state` - Reference to `AppState` for workspace-specific paths.
/// * `target` - The `ConfigurationTarget` (User, Workspace, WorkspaceFolder).
/// * `overrides` - `IConfigurationOverrides` containing optional resource URI
///   for `WORKSPACE_FOLDER` target.
/// * `_scope_to_language` - Boolean indicating if the update is
///   language-specific (currently unused).
///
/// # Returns
/// * `Ok(PathBuf)` with the absolute path to the settings file.
/// * `Err(CommonError)` if path resolution fails or target is unsupported.
pub fn get_config_path_for_target<R:TauriRuntime>(
	app_handle:&AppHandle<R>,

	app_state:&AppState,

	target:ConfigurationTarget,

	overrides:&IConfigurationOverrides,

	// Unused in MVP path resolution
	_scope_to_language:bool,
) -> Result<PathBuf, CommonError> {
	trace!(
		"[ConfigHelpers Path] Resolving config path: target={:?}, overrides.resource={:?}, overrides.langId={:?}",
		target,
		overrides.resource.as_ref().and_then(|v| v.get("external")),
		// languageId
		overrides.override_identifier
	);

	let path_resolver = app_handle.path_resolver();

	// Base directory for user-level settings (e.g., ~/.config/YourApp/User/)
	let base_user_config_dir = path_resolver.app_config_dir().ok_or_else(|| {
		create_config_error(
			"path_resolution",
			"Cannot resolve app config directory (for User settings)".to_string(),
		)
	})?;

	match target {
		ConfigurationTarget::USER_LOCAL | ConfigurationTarget::USER => {
			// VS Code often has settings directly in AppData/User/settings.json
			// or AppData/User/profiles/<profile>/settings.json
			// For simplicity, using AppData/User/settings.json
			// TODO: Add support for profiles if that feature is planned.
			let user_settings_path = base_user_config_dir.join("User").join("settings.json");

			debug!(
				"[ConfigHelpers Path] Resolved User settings path: {}",
				user_settings_path.display()
			);

			Ok(user_settings_path)
		},

		ConfigurationTarget::WORKSPACE => {
			let config_path_guard = app_state
				.workspace_config_path
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			let ws_path = config_path_guard.as_ref().cloned().ok_or_else(|| {
				create_config_error(
					"path_resolution",
					"No workspace configuration file (.code-workspace) loaded. Cannot target WORKSPACE settings."
						.to_string(),
				)
			})?;

			debug!(
				"[ConfigHelpers Path] Resolved Workspace settings path (from .code-workspace): {}",
				ws_path.display()
			);

			Ok(ws_path)
		},

		ConfigurationTarget::WORKSPACE_FOLDER => {
			let resource_uri_val = overrides.resource.as_ref().ok_or_else(|| {
				create_config_error(
					"path_resolution",
					"Missing resource URI for WORKSPACE_FOLDER target".to_string(),
				)
			})?;

			// Attempt to parse the resource URI from the override
			let resource_uri_str = resource_uri_val
				.get("external")
				.and_then(Value::as_str)
				.or_else(|| {
					// Fallback to 'path' if 'external' is missing
					resource_uri_val.get("path").and_then(Value::as_str).map(|p_str| {
						if Path::new(p_str).is_absolute() {
							Url::from_file_path(p_str)
								.map(|u| u.to_string())
								 // Fallback to raw string if path->URL fails
								.unwrap_or_else(|_| p_str.to_string())
						} else {
							// Assume it's a scheme or opaque URI
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

			let resource_uri = Url::parse(resource_uri_str).map_err(|e| {
				CommonError::InvalidArg(
					"resource".to_string(),
					format!("Invalid resource URI in overrides: '{}', error: {}", resource_uri_str, e),
				)
			})?;

			let folders_guard = app_state
				.workspace_folders
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			let containing_folder = folders_guard
				.iter()
				.find(|f| {
					// Check scheme and if the resource URI's path starts with the folder's path
					resource_uri.scheme() == f.uri.scheme() && resource_uri.path().starts_with(f.uri.path())
				})
				.ok_or_else(|| {
					create_config_error(
						"path_resolution",
						format!("Resource URI '{}' does not belong to any known workspace folder.", resource_uri),
					)
				})?;

			if containing_folder.uri.scheme() != "file" {
				return Err(create_config_error(
					"path_resolution",
					format!(
						"Cannot get folder settings for non-file scheme folder: {}",
						containing_folder.uri
					),
				));
			}

			// Path to .vscode/settings.json within the folder
			let folder_settings_path = PathBuf::from(containing_folder.uri.path())
				.join(".vscode")
				.join("settings.json");

			debug!(
				"[ConfigHelpers Path] Resolved Workspace Folder settings path: {}",
				folder_settings_path.display()
			);

			Ok(folder_settings_path)
		},

		_ => {
			// Other targets like MEMORY, POLICY are not handled for direct file path
			// resolution
			warn!(
				"[ConfigHelpers Path] ConfigurationTarget {:?} path resolution not implemented for general settings \
				 update.",
				target
			);

			Err(CommonError::NotImplemented(format!(
				"ConfigurationTarget {:?} path resolution not implemented.",
				target
			)))
		},
	}
}

/// Asynchronously loads JSON content from a file.
///
/// If the file does not exist or is empty, it returns a default empty JSON
/// object (`{}`).
///
/// # Argument
/// * `path` - The `Path` to the JSON file.
///
/// # Returns
/// * `Ok(Value)` with the parsed JSON data or an empty object.
/// * `Err(CommonError)` if file reading or JSON parsing fails (other than
///   NotFound).
pub async fn load_json_file_if_exists_or_default(path:&Path) -> Result<Value, CommonError> {
	trace!("[ConfigHelpers IO] Attempting to load JSON from: {}", path.display());

	match fs::read_to_string(path).await {
		Ok(content) => {
			if content.trim().is_empty() {
				debug!(
					"[ConfigHelpers IO] File is empty, returning default empty JSON object: {}",
					path.display()
				);

				// Default to empty object if file is empty
				Ok(json!({}))
			} else {
				serde_json::from_str(&content).map_err(|e| {
					create_config_error("load", format!("JSON parse failed for {}: {}", path.display(), e))
				})
			}
		},

		Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
			debug!(
				"[ConfigHelpers IO] File not found, returning default empty JSON object: {}",
				path.display()
			);

			// Default to empty object if not found
			Ok(json!({}))
		},

		Err(e) => {
			Err(create_config_error(
				"load",
				format!("File read failed for {}: {}", path.display(), e),
			))
		},
	}
}

/// Asynchronously writes a `serde_json::Value` to a file, pretty-printed.
///
/// Creates parent directories if they don't exist.
///
/// # Argument
/// * `path` - The `Path` to write the JSON file to.
/// * `value` - The `serde_json::Value` to serialize and write.
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(CommonError)` if directory creation, serialization, or file writing
///   fails.
pub async fn write_json_file(path:&Path, value:&Value) -> Result<(), CommonError> {
	trace!("[ConfigHelpers IO] Attempting to write JSON to: {}", path.display());

	let parent = path
		.parent()
		.ok_or_else(|| create_config_error("write", format!("Invalid path (has no parent): {}", path.display())))?;

	// Check if parent directory exists, create if not
	if !fs::try_exists(parent).await.map_err(|e| {
		create_config_error(
			"create_dir_check",
			format!("Failed to check existence of {}: {}", parent.display(), e),
		)
	})? {
		debug!(
			"[ConfigHelpers IO] Creating parent directory for config file: {}",
			parent.display()
		);

		fs::create_dir_all(parent).await.map_err(|e| {
			create_config_error("create_dir", format!("Failed to create directory {}: {}", parent.display(), e))
		})?;
	}

	let content = serde_json::to_string_pretty(value).map_err(|e| {
		create_config_error("serialize", format!("Failed to serialize JSON for {}: {}", path.display(), e))
	})?;

	// Use tokio::fs::File for async writing
	let mut file = fs::File::create(path).await.map_err(|e| {
		create_config_error("write", format!("Failed to create/open for writing {}: {}", path.display(), e))
	})?;

	file.write_all(content.as_bytes())
		.await
		.map_err(|e| create_config_error("write", format!("Failed to write to {}: {}", path.display(), e)))?;

	info!("[ConfigHelpers IO] Successfully wrote JSON to {}", path.display());

	Ok(())
}

/// Updates a value in a `serde_json::Value` (assumed to be an Object) at a
/// given dot-separated key path.
///
/// If `value_to_set` is `Value::Null`, the key at the specified path is
/// removed. Intermediate objects along the path are created if they don't exist
/// when setting a value.
///
/// # Argument
/// * `target_value` - A mutable reference to the `serde_json::Value` to update.
/// * `key_path` - A dot-separated string representing the path to the key
///   (e.g., "editor.fontSize").
/// * `value_to_set` - The `serde_json::Value` to set at the key path. If
///   `Null`, the key is removed.
pub fn update_json_value_at_path(target_value:&mut Value, key_path:&str, value_to_set:Value) {
	trace!(
		"[ConfigHelpers Json] Updating value at path: '{}', new value (is_null: {})",
		key_path,
		value_to_set.is_null()
	);

	let mut current = target_value;

	let parts:Vec<&str> = key_path.split('.').collect();

	if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
		warn!(
			"[ConfigHelpers Json] Attempted to update with empty or invalid key path: '{}'.",
			key_path
		);

		// If key_path is empty and target is an object, replace the whole object
		// if value_to_set is also an object.
		if key_path.is_empty() && current.is_object() && !value_to_set.is_null() && value_to_set.is_object() {
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

			// Skip empty segments
			continue;
		}

		if i == last_part_index {
			// At the final key segment
			if let Some(obj) = current.as_object_mut() {
				if value_to_set.is_null() {
					trace!("[ConfigHelpers Json] Removing key: '{}'", part_key);

					obj.remove(&part_key);
				} else {
					trace!(
						"[ConfigHelpers Json] Setting key: '{}' to value (type: {:?})",
						part_key,
						value_to_set.kind()
					);

					obj.insert(part_key, value_to_set);
				}
			} else {
				warn!(
					"[ConfigHelpers Json] Cannot set key '{}' in path '{}' because parent is not an object (current \
					 type: {:?}).",
					part_key,
					key_path,
					current.kind()
				);
			}

			// Operation complete or failed at last part
			return;
		} else {
			// Traversing or creating intermediate objects
			if !current.is_object() {
				// Current part is not an object, but we need to go deeper.
				// Overwrite current with an empty object to proceed.
				debug!(
					"[ConfigHelpers Json] Path segment '{}' in '{}' is not an object (is {:?}), creating new object \
					 node.",
					part_key,
					key_path,
					current.kind()
				);

				*current = json!({});
			}

			// Now `current` is guaranteed to be an object.
			current = current
				.as_object_mut()
				 // Should not panic due to the check above
				.unwrap()
				.entry(part_key)
				 // Create intermediate object if it doesn't exist
				.or_insert_with(|| json!({}));
		}
	}
}

/// Loads configurations from user, workspace, and workspace folder settings,
///
///
/// and performs a simplified merge.
///
/// The merge order is: User -> Workspace (.code-workspace `settings` key) ->
/// Workspace Folder (`.vscode/settings.json`). Later sources override earlier
/// ones.
///
/// # Argument
/// * `app_handle` - Tauri `AppHandle`.
/// * `app_state` - Reference to `AppState`.
///
/// # Returns
/// * `Ok(ConfigurationState)` with the merged configuration.
/// * `Err(CommonError)` if any configuration loading or merging step fails.
pub async fn load_and_merge_configurations_internal<R:TauriRuntime>(
	app_handle:&AppHandle<R>,

	app_state:&AppState,
) -> Result<ConfigurationState, CommonError> {
	info!("[ConfigHelpers Merge] Loading and merging all configurations...");

	// 1. Load User settings
	let user_config_path = get_config_path_for_target(
		app_handle,
		app_state,
		// Or USER, effectively same for file path
		ConfigurationTarget::USER_LOCAL,
		// No overrides for user general
		&IConfigurationOverrides::default(),
		// Not language-scoped for general user
		false,
	)?;

	let mut merged_config_data = load_json_file_if_exists_or_default(&user_config_path).await?;

	debug!(
		"[ConfigHelpers Merge] Loaded user config ({} top-level keys) from: {}",
		merged_config_data.as_object().map_or(0, Map::len),
		user_config_path.display()
	);

	// 2. Load Workspace settings (from .code-workspace file, if any)
	let workspace_config_path_opt = app_state
		.workspace_config_path
		.lock()
		.map_err(map_app_state_lock_error_to_common_error)?
		.clone();

	if let Some(ws_config_path) = workspace_config_path_opt {
		// Check if the .code-workspace file itself exists and is a file
		if fs::try_exists(&ws_config_path).await.unwrap_or(false) && ws_config_path.is_file() {
			let workspace_file_values = load_json_file_if_exists_or_default(&ws_config_path).await?;

			// Workspace settings are typically under a "settings" key in the
			// .code-workspace file
			if let Some(settings_in_workspace_file) = workspace_file_values.get("settings") {
				if settings_in_workspace_file.is_object() {
					debug!(
						"[ConfigHelpers Merge] Loaded workspace settings ({} top-level keys) from .code-workspace: {}",
						settings_in_workspace_file.as_object().map_or(0, Map::len),
						ws_config_path.display()
					);

					merge_json_values(&mut merged_config_data, settings_in_workspace_file);
				} else {
					warn!(
						"[ConfigHelpers Merge] 'settings' key in {} is not an object. Skipping workspace settings.",
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
				"[ConfigHelpers Merge] Workspace config path {} does not exist or is not a file. Skipping.",
				ws_config_path.display()
			);
		}
	} else {
		trace!("[ConfigHelpers Merge] No .code-workspace file configured.");
	}

	// 3. Load Workspace Folder settings (from each folder's .vscode/settings.json)
	let folders_guard = app_state
		.workspace_folders
		.lock()
		.map_err(map_app_state_lock_error_to_common_error)?;

	for folder_state in folders_guard.iter() {
		if folder_state.uri.scheme() == "file" {
			let folder_path = PathBuf::from(folder_state.uri.path());

			let folder_settings_path = folder_path.join(".vscode").join("settings.json");

			if fs::try_exists(&folder_settings_path).await.unwrap_or(false) {
				let folder_values = load_json_file_if_exists_or_default(&folder_settings_path).await?;

				debug!(
					"[ConfigHelpers Merge] Loaded folder settings ({} top-level keys) for '{}' from: {}",
					folder_values.as_object().map_or(0, Map::len),
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

	// Release lock
	drop(folders_guard);

	info!(
		"[ConfigHelpers Merge] Configuration merge complete. Effective top-level keys: {}",
		merged_config_data.as_object().map_or(0, Map::len)
	);

	trace!(
		"[ConfigHelpers Merge] Final merged data (sample): {}",
		merged_config_data
			.to_string()
			.chars()
			 // Log a sample
			.take(200)
			.collect::<String>()
	);

	Ok(ConfigurationState::new(merged_config_data))
}

/// Performs a simple recursive merge of two `serde_json::Value`s.
///
/// Values from `source` override corresponding values in `target`.
/// - If both `target` and `source` are objects at a given key, their fields are
///   merged recursively.
/// - If `source` contains an array, it replaces the `target`'s value at that
///   key (arrays are not merged element-wise).
/// - If `source` contains a null, it replaces the `target`'s value.
///
/// # Argument
/// * `target` - A mutable reference to the `Value` to be merged into.
/// * `source` - A reference to the `Value` whose contents will override
///   `target`.
fn merge_json_values(target:&mut Value, source:&Value) {
	match (target, source) {
		(Value::Object(target_map), Value::Object(source_map)) => {
			for (key, source_val) in source_map {
				// Get or insert the value from source if key doesn't exist in target
				let target_entry = target_map.entry(key.clone()).or_insert_with(|| source_val.clone());

				// If both are objects, recurse. Otherwise, source_val (cloned into
				// target_entry if new, or explicit assignment below) takes precedence.
				if target_entry.is_object() && source_val.is_object() {
					merge_json_values(target_entry, source_val);
				} else {
					// Source overrides target if not both objects or if target_entry was just
					// inserted
					*target_entry = source_val.clone();
				}
			}
		},

		(target_val, source_val) => {
			// For non-object types or mismatched types, source completely overrides target.
			*target_val = source_val.clone();
		},
	}
}

/// Notifies Cocoon (via Vine) that specified configuration keys have changed.
///
/// The notification payload includes the full effective configuration and a
/// list of affected keys, matching VS Code's `$acceptConfigurationChanged`
/// protocol.
///
/// # Argument
/// * `app_handle` - Tauri `AppHandle`.
/// * `affected_keys` - A `Vec<String>` of dot-separated configuration keys that
///   have changed.
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
			.map_err(map_app_state_lock_error_to_common_error)
			// Panicking here because if config state is unrecoverable, app is likely broken.
			.expect("Config lock failed for notification; AppState potentially poisoned.");

		let effective_config = config_guard.data.clone();

		// Construct IConfigurationInitData DTO (simplified for MVP)
		// TODO: Populate 'defaults', 'user', 'workspace', 'folders' with actual content
		//       if/when Mountain has a more sophisticated multi-file config model.
		//       For now, only 'effective' is crucial for Cocoon's immediate needs.
		let config_init_data_dto = json!({




			"effective": effective_config,


			 // Stub
			"defaults": { "contents": {} },


			 // Stub
			"user": { "contents": {} },


			"workspace": { "contents": {} },// Stub
			 // Stub
			"folders": [],


			 // Stub
			"memory": { "contents": {} },


			 // Stub
			"policy": Value::Null,


			 // Stub
			"configurationScopes": []
		});

		// Construct IConfigurationChangeEvent DTO
		let change_event_dto = json!({




			"keys": affected_keys,


			// TODO: Populate 'overrides' if the change affects specific resources/languages
			 // Stub
			"overrides": []
		});

		(config_init_data_dto, change_event_dto)
	};

	// Payload for $acceptConfigurationChanged is [IConfigurationInitData,

	// IConfigurationChangeEvent]
	let payload_to_send = json!([config_init_data_dto, change_event_dto]);

	trace!(
		"[ConfigHelpers Notify] $acceptConfigurationChanged payload (brief): keys: {:?}, effective keys: {}",
		affected_keys,
		payload_to_send[0]["effective"].as_object().map_or(0, Map::len)
	);

	if let Err(e) = vine::send_notification_to_sidecar(
		// TODO: Make sidecar ID configurable if multiple sidecars can exist
		"cocoon-main",
		"$acceptConfigurationChanged".to_string(),
		payload_to_send,
	)
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

/// DTO for deserializing a `DocumentFilter` from `serde_json::Value`.
/// Mirrors `vscode.DocumentFilter`.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFilterDto {
	pub language:Option<String>,

	pub scheme:Option<String>,

	pub pattern:Option<String>,

	/// `UriComponents` JSON Value, e.g., `{"scheme":"file",
	///
	///
	/// "path":"/base/dir"}`
	pub base_uri:Option<Value>,
}

/// Matches a document against a `DocumentSelector` (which can be a string, an
/// array of filters, or a single filter object).
///
/// Uses `globset` for pattern matching.
///
/// # Argument
/// * `selector_val` - A `serde_json::Value` representing the
///   `DocumentSelector`.
/// * `document_uri` - The `Url` of the document to match.
/// * `language_id` - The language ID string of the document.
///
/// # Returns
/// `true` if the document matches the selector, `false` otherwise.
///
/// # Current Limitations & TODOs for Glob Patterns:
/// - **Base URI Handling:** If `filter.base_uri` is present in a
///   `DocumentFilterDto`, the `pattern` should ideally be resolved relative to
///   this base URI before matching against a path derived from `document_uri`.
///   This is not yet fully implemented; patterns are matched against the full
///   document path.
/// - **Workspace-Relative Globs:** If a `pattern` is relative (e.g.,
///
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
		// More descriptive for non-string
		selector_val.as_str().unwrap_or("<object/array selector>"),
		document_uri.as_str(),
		language_id
	);

	// Case 1: Selector is a simple language ID string (or "*")
	if let Some(selector_lang_id_str) = selector_val.as_str() {
		let matches = selector_lang_id_str == language_id || selector_lang_id_str == "*";

		trace!(
			"[SelectorMatch] String selector '{}' vs lang '{}': {}",
			selector_lang_id_str, language_id, matches
		);

		return matches;
	}

	// Case 2: Selector is an array of filters (DocumentFilterDto or language ID
	// string)
	if let Some(selector_array) = selector_val.as_array() {
		// Any one filter in the array must match for the selector to match
		let matches = selector_array
			.iter()
			.any(|s_filter| match_document_selector(s_filter, document_uri, language_id));

		trace!("[SelectorMatch] Array selector result: {}", matches);

		return matches;
	}

	// Case 3: Selector is a DocumentFilterDto object
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

				trace!("[SelectorMatch] Language check passed (or no language filter).");

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

				trace!("[SelectorMatch] Scheme check passed (or no scheme filter).");

				// 3. Pattern Check (Glob)
				if let Some(filter_pattern_str) = &filter.pattern {
					// TODO: Implement full base_uri relative glob matching.
					//       If `filter.base_uri` is Some, `filter_pattern_str` should be
					//       interpreted relative to that base. The `document_uri.path()`
					//       would then need to be made relative to that same base before matching.
					//       This requires careful path arithmetic.
					if filter.base_uri.is_some() {
						warn!(
							"[SelectorMatch] Glob pattern '{}' has a 'base_uri' specified, but base URI resolution \
							 for globs is NOT YET IMPLEMENTED. Pattern will be matched against the full document path.",
							filter_pattern_str
						);

						// For now, ignore base_uri and match pattern against
						// full document_uri.path()
					}

					// TODO: Implement workspace-folder-relative glob matching if no base_uri.
					//       This would involve iterating app_state.workspace_folders, making
					//       document_uri.path() relative to each folder root, and trying to match.

					// Full, percent-decoded path
					let path_to_match = document_uri.path();

					trace!(
						"[SelectorMatch] Glob: pattern='{}', attempting match against path='{}'",
						filter_pattern_str, path_to_match
					);

					let mut glob_builder = GlobBuilder::new(filter_pattern_str);

					// Case insensitivity should match filesystem behavior (e.g., true on Windows)
					glob_builder.case_insensitive(cfg!(windows));

					// On Windows, `\` can be a literal or separator. `false` means `\` is a
					// separator. `globset` generally expects POSIX-style paths in patterns.
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

							// Invalid glob means no match
							return false;
						},
					}
				}

				trace!("[SelectorMatch] Pattern check passed (or no pattern filter).");

				// If all checks passed
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

	// Fallback for unrecognized selector format
	warn!(
		"[SelectorMatch] Unrecognized selector format (not string, array, or filter object), no match: {:?}",
		selector_val
	);

	false
}
