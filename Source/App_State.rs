// ---------------------------------------------------------------------------------------------
// Mountain Application State 
// --------------------------------------------------------------------------------------------
// Defines the central `AppState` struct managed by Tauri via `app.manage()`.
// This struct aggregates all shared, mutable application state required across
// different parts of Mountain, including command handlers, effect
// implementations (e.g., `MountainEnvironment`), and background tasks. State is
// wrapped appropriately (e.g., `Arc<StdMutex<_>>`, `Arc<AtomicBool>`) for
// thread-safe access from both synchronous and asynchronous contexts.
//
// Responsibilities:
// - Defining the structure for all shared application state data.
// - Providing a `Default` implementation (`AppState::default()`) for
//   initialization.
// - Providing helper methods for common state-related operations.
//
// Key Interactions:
// - Instantiated once and managed by Tauri.
// - Accessed throughout the application via `app_handle.state::<AppState>()`.
// - Its fields are read from and written to (under locks) by various handlers
//   and effect implementations.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	fs, // Used for synchronous I/O during initialization and current `scan_extensions`
	path::{Path, PathBuf},
	sync::{
		Arc,
		Mutex as StdMutex, // Standard Mutex for thread-safe interior mutability
		atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering as AtomicOrdering},
	},
};

// Land_Common imports
use Land_Common::{
	config_effects::ConfigurationScope, // For MergedConfigurationState.get_all_configuration_scopes_for_rpc
	errors::CommonError,                // For PendingUiRequestChannelMap's Result type
	language_feature_effects::{ProviderOptionsDto, ProviderType as CommonLangProviderType}, // For ProviderRegistration
};
// Logging
use log::{debug, error, info, trace, warn};
// For serializing/deserializing state DTOs
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
// Tauri essentials
use tauri::{Manager, Wry};
use tokio::{
	sync::{
		Mutex as TokioMutex,     // Tokio Mutex for JoinHandle wrappers in TerminalState
		mpsc as TokioMpsc,       // Tokio MPSC for terminal input channel sender
		oneshot as TokioOneshot, // Tokio oneshot for pending UI requests
	},
	task::JoinHandle, // For storing handles to spawned terminal tasks
};
// For URI handling in various state DTOs
use url::Url;

use crate::handlers::{
	commands::{self, CommandHandler}, // Enum for native/proxied command handlers
	diagnostics::MarkerData,          // DTO for diagnostic markers
};

// --- Type Aliases for Clarity ---
pub type CommandRegistryMap = HashMap<String, CommandHandler<Wry>>;
pub type DiagnosticsStorageMap = HashMap<String /* owner */, HashMap<String /* UriString */, Vec<MarkerData>>>;
pub type MementoStorageMap = HashMap<String /* key */, Value /* value */>;
pub type OpenDocumentMap = HashMap<String /* UriString (key) */, DocumentState>;
pub type OutputChannelStorageMap = HashMap<String /* Channel ID */, OutputChannelState>;
pub type LanguageProviderRegistrationMap = HashMap<u32 /* Handle */, ProviderRegistration>;
pub type ScannedExtensionMetadataMap =
	HashMap<String /* Extension ID (publisher.name) */, ExtensionDescriptionState>;
pub type EnabledProposedApisConfigMap = HashMap<String /* Extension ID or '*' */, Vec<String /* proposal name */>>;
pub type ActiveTerminalMap = HashMap<u64 /* Terminal ID */, Arc<StdMutex<TerminalState>>>;
pub type PendingUiRequestChannelMap =
	HashMap<String /* Request ID */, TokioOneshot::Sender<Result<Value, CommonError>>>;
pub type ActiveHierarchySessionMap = HashMap<String /* Cocoon's SessionID */, HierarchySessionContext>;

// --- State Structures (DTOs for complex state fields) ---

/// Represents the state of a single workspace folder.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkspaceFolderState {
	#[serde(with = "url_serde_helper")] // Custom serializer for url::Url
	pub uri: Url, // URI of the folder (e.g., "file:///path/to/folder")
	pub name:String, // Display name of the folder
	pub index:usize, // Order/index of the folder within the workspace
}

/// Represents the merged configuration state of the application.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MergedConfigurationState {
	/// Holds the effective, merged configuration values from all sources.
	pub data:Value,
}

impl MergedConfigurationState {
	pub fn new(data:Value) -> Self { Self { data } }

	pub fn get_value(&self, section:Option<&str>, _scope_uri_components:Option<&Value>) -> Value {
		trace!(
			"[AppState ConfigAccess] get_value: section={:?}, scope_uri_components={:?}",
			section, _scope_uri_components
		);
		if let Some(s_path) = section {
			let mut current_val = &self.data;
			for part_key in s_path.split('.') {
				if let Some(next_val) = current_val.get(part_key) {
					current_val = next_val;
				} else {
					trace!(
						"[AppState ConfigAccess] Section part '{}' not found for path '{}'. Returning null.",
						part_key, s_path
					);
					return Value::Null;
				}
			}
			current_val.clone()
		} else {
			self.data.clone()
		}
	}

	pub fn update_from_new_state(&mut self, new_state:MergedConfigurationState) {
		info!("[AppState ConfigAccess] Updating entire merged configuration state.");
		trace!(
			"[AppState ConfigAccess] Old data items: {}, New data items: {}",
			self.data.as_object().map_or(0, |m| m.len()),
			new_state.data.as_object().map_or(0, |m| m.len())
		);
		self.data = new_state.data;
	}

	/// Returns the configuration scopes for all known keys.
	/// This is a simplified version. A real implementation would parse
	/// package.json contributions for all known extensions and core settings
	/// to determine their declared scopes.
	pub fn get_all_configuration_scopes_for_rpc(&self) -> Vec<(String, ConfigurationScope)> {
		let mut scopes = Vec::new();
		if let Some(obj_map) = self.data.as_object() {
			for key in obj_map.keys() {
				// Default to Window scope if not otherwise known.
				// This is a placeholder; actual scopes must be determined from contributions.
				let scope = if key.starts_with("files.") || key.starts_with("search.") {
					ConfigurationScope::Resource // Example: files.eol, search.exclude are often resource-scoped
				} else if key.starts_with("workbench.") || key.starts_with("editor.") {
					ConfigurationScope::Window // Example: workbench.colorTheme, editor.fontSize
				} else {
					// Fallback for user/global settings or unknown settings.
					// Many settings defined at the root without specific prefixes might be
					// considered Application or Window. For safety and commonality, Window is
					// a reasonable default if not Application.
					ConfigurationScope::Window
				};
				scopes.push((key.clone(), scope));
			}
		}
		if scopes.is_empty() && self.data.is_object() && !self.data.as_object().unwrap().is_empty() {
			warn!(
				"[AppState Config] get_all_configuration_scopes_for_rpc: No specific scopes derived for effective \
				 keys. Returning keys with default Window scope."
			);
		} else if self.data.is_object() && self.data.as_object().unwrap().is_empty() {
			debug!(
				"[AppState Config] get_all_configuration_scopes_for_rpc: No effective configuration data to derive \
				 scopes from."
			);
		}
		trace!("[AppState Config] Derived scopes for RPC: {:?}", scopes);
		scopes
	}
}

/// Represents a content change operation on a text model (from Cocoon).
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RpcModelContentChangeDto {
	range:RpcRangeDto,
	text:String,
}

/// Represents a range in a text model (0-indexed from Cocoon).
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RpcRangeDto {
	start_line_number:usize,
	start_column:usize,
	end_line_number:usize,
	end_column:usize,
}

/// Represents the state of an open text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentState {
	#[serde(with = "url_serde_helper")]
	pub uri:Url,
	pub language_id:String,
	pub version:i64,
	pub lines:Vec<String>,
	pub eol:String,
	pub is_dirty:bool,
	pub encoding:String,
}

impl DocumentState {
	pub fn get_text_content(&self) -> String { self.lines.join(&self.eol) }

	pub fn apply_document_content_changes(&mut self, new_version_id:i64, changes_dto_val:&Value) -> Result<(), String> {
		if new_version_id <= self.version && changes_dto_val.as_array().map_or(false, |arr| !arr.is_empty()) {
			warn!(
				"[DocState ApplyChanges] Ignoring stale changes for {}: V{} <= Current V{}. Changes: {:?}",
				self.uri, new_version_id, self.version, changes_dto_val
			);
			return Ok(());
		}
		if new_version_id <= self.version && changes_dto_val.as_array().map_or(true, |arr| arr.is_empty()) {
			debug!(
				"[DocState ApplyChanges] Ignoring stale/no-op version bump for {}: V{} <= Current V{}.",
				self.uri, new_version_id, self.version
			);
			return Ok(());
		}
		debug!(
			"[DocState ApplyChanges] Applying V{} (Current V{}) for {}. Changes: {:?}",
			new_version_id, self.version, self.uri, changes_dto_val
		);

		let rpc_changes_vec:Vec<RpcModelContentChangeDto> = match serde_json::from_value(changes_dto_val.clone()) {
			Ok(changes) => changes,
			Err(deser_error) => {
				if let Some(full_text_str) = changes_dto_val.as_str() {
					info!(
						"[DocState ApplyChanges] Full text replacement for V{} on {}.",
						new_version_id, self.uri
					);
					let (new_lines, new_eol) = analyze_text_lines_and_eol_for_document_state(full_text_str);
					self.lines = new_lines;
					self.eol = new_eol;
					self.version = new_version_id;
					self.is_dirty = true;
					return Ok(());
				}
				if changes_dto_val.as_array().map_or(true, |arr| arr.is_empty()) && new_version_id > self.version {
					debug!(
						"[DocState ApplyChanges] Version bump (V{} -> V{}) with no content changes for {}.",
						self.version, new_version_id, self.uri
					);
					self.version = new_version_id;
					return Ok(());
				}
				return Err(format!("Invalid RpcModelContentChangeDto for {}: {}", self.uri, deser_error));
			},
		};

		if rpc_changes_vec.is_empty() && new_version_id > self.version {
			debug!(
				"[DocState ApplyChanges] Version bump (V{} -> V{}) with empty changes array for {}.",
				self.version, new_version_id, self.uri
			);
			self.version = new_version_id;
			return Ok(());
		}

		for change_op in rpc_changes_vec {
			let mut start_line_idx = change_op.range.start_line_number;
			let mut start_col_char_idx = change_op.range.start_column;
			let mut end_line_idx = change_op.range.end_line_number;
			let mut end_col_char_idx = change_op.range.end_column;

			trace!(
				"[DocState ApplyChanges] Single change: L{}(C{})-L{}(C{}), text: '{}...'",
				start_line_idx + 1,
				start_col_char_idx + 1,
				end_line_idx + 1,
				end_col_char_idx + 1,
				change_op.text.chars().take(20).collect::<String>()
			);

			if start_line_idx > self.lines.len() || end_line_idx > self.lines.len() {
				error!(
					"[DocState ApplyChanges] Invalid line range for {}: L{}-L{} > lines {}. Change: {:?}. Skipping.",
					self.uri,
					start_line_idx + 1,
					end_line_idx + 1,
					self.lines.len(),
					change_op
				);
				continue;
			}
			if start_line_idx < self.lines.len() {
				start_col_char_idx = std::cmp::min(start_col_char_idx, self.lines[start_line_idx].chars().count());
			} else if start_line_idx == self.lines.len() && start_col_char_idx != 0 {
				error!(
					"[DocState ApplyChanges] Invalid start col for append {}: L{}, Col{}. Change: {:?}. Skipping.",
					self.uri,
					start_line_idx + 1,
					start_col_char_idx + 1,
					change_op
				);
				continue;
			}
			if end_line_idx < self.lines.len() {
				end_col_char_idx = std::cmp::min(end_col_char_idx, self.lines[end_line_idx].chars().count());
			} else if end_line_idx == self.lines.len() && end_col_char_idx != 0 {
				error!(
					"[DocState ApplyChanges] Invalid end col for new line {}: L{}, Col{}. Change: {:?}. Skipping.",
					self.uri,
					end_line_idx + 1,
					end_col_char_idx + 1,
					change_op
				);
				continue;
			}

			let text_to_insert_lines:Vec<String> = change_op.text.split(&self.eol).map(String::from).collect();

			if start_line_idx == end_line_idx {
				// Single-line change
				if start_line_idx >= self.lines.len() {
					// Append new lines at end
					if start_line_idx == self.lines.len() && start_col_char_idx == 0 && end_col_char_idx == 0 {
						self.lines.extend(text_to_insert_lines);
					} else {
						error!(
							"[DocState ApplyChanges] Single-line change on non-existent line or invalid cols for {}. \
							 Change: {:?}. Skipping.",
							self.uri, change_op
						);
						continue;
					}
				} else {
					// Modify existing line
					let line_to_modify = &mut self.lines[start_line_idx];
					let original_line_tail:String = line_to_modify.chars().skip(end_col_char_idx).collect();
					let mut new_line_content:String = line_to_modify.chars().take(start_col_char_idx).collect();
					if text_to_insert_lines.len() == 1 {
						new_line_content.push_str(&text_to_insert_lines[0]);
						new_line_content.push_str(&original_line_tail);
						*line_to_modify = new_line_content;
					} else {
						// Inserted text is multi-line, splitting current line
						new_line_content.push_str(&text_to_insert_lines[0]);
						*line_to_modify = new_line_content;
						for i in 1..text_to_insert_lines.len() - 1 {
							self.lines.insert(start_line_idx + i, text_to_insert_lines[i].clone());
						}
						let last_inserted_line_part = text_to_insert_lines.last().unwrap().clone();
						self.lines.insert(
							start_line_idx + text_to_insert_lines.len() - 1,
							last_inserted_line_part + &original_line_tail,
						);
					}
				}
			} else {
				// Multi-line change
				if start_line_idx >= self.lines.len() {
					error!(
						"[DocState ApplyChanges] Multi-line change on non-existent line {} for {}. Change: {:?}. \
						 Skipping.",
						start_line_idx + 1,
						self.uri,
						change_op
					);
					continue;
				}
				let first_line_prefix:String = self.lines[start_line_idx].chars().take(start_col_char_idx).collect();
				let last_line_suffix:String = if end_line_idx < self.lines.len() {
					self.lines[end_line_idx].chars().skip(end_col_char_idx).collect()
				} else {
					String::new()
				};
				let mut modified_start_line_content = first_line_prefix;
				modified_start_line_content.push_str(&text_to_insert_lines[0]);
				if text_to_insert_lines.len() == 1 {
					modified_start_line_content.push_str(&last_line_suffix);
					self.lines[start_line_idx] = modified_start_line_content;
				} else {
					self.lines[start_line_idx] = modified_start_line_content;
					for i in 1..text_to_insert_lines.len() - 1 {
						self.lines.insert(start_line_idx + i, text_to_insert_lines[i].clone());
					}
					let final_inserted_line_content = text_to_insert_lines.last().unwrap().clone() + &last_line_suffix;
					self.lines
						.insert(start_line_idx + text_to_insert_lines.len() - 1, final_inserted_line_content);
				}
				let num_original_lines_in_deleted_range_after_start_line = end_line_idx - start_line_idx;
				if num_original_lines_in_deleted_range_after_start_line > 0 {
					let removal_start_actual_idx = start_line_idx + std::cmp::max(1, text_to_insert_lines.len());
					if removal_start_actual_idx < self.lines.len() {
						let removal_end_actual_idx = std::cmp::min(
							self.lines.len(),
							removal_start_actual_idx + num_original_lines_in_deleted_range_after_start_line,
						);
						if removal_start_actual_idx < removal_end_actual_idx {
							self.lines.drain(removal_start_actual_idx..removal_end_actual_idx);
						}
					} else if removal_start_actual_idx == self.lines.len()
						&& num_original_lines_in_deleted_range_after_start_line > 0
					{
						trace!(
							"[DocState ApplyChanges] Multi-line delete range at/beyond end after insertions for {}. \
							 No lines drained.",
							self.uri
						);
					} else if num_original_lines_in_deleted_range_after_start_line > 0 {
						debug!(
							"[DocState ApplyChanges] Calculated removal start index {} out of bounds (lines: {}). \
							 URI: {}",
							removal_start_actual_idx,
							self.lines.len(),
							self.uri
						);
					}
				}
			}
		}
		self.version = new_version_id;
		self.is_dirty = true;
		Ok(())
	}
}

/// Represents the state of an active integrated terminal.
#[derive(Debug, Clone)]
pub struct TerminalState {
	pub id:u64,
	pub name:String,
	pub shell_path:String,
	pub shell_args:Vec<String>,
	pub cwd:Option<PathBuf>,
	pub env:Option<HashMap<String, String>>,
	pub os_process_id:Option<u32>,
	pub is_pty:bool,
	#[serde(skip)]
	pub pty_input_tx:Option<TokioMpsc::Sender<String>>,
	#[serde(skip)]
	pub reader_task_handle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
	#[serde(skip)]
	pub process_wait_handle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
}

impl TerminalState {
	pub fn new(id:u64, name:String, options_val:&Value, default_shell_path:String) -> Self {
		let shell_path_opt_str = options_val.get("shellPath").and_then(Value::as_str);
		let final_shell_path = shell_path_opt_str.map_or(default_shell_path, String::from);
		let shell_args_val = options_val.get("shellArgument");
		let final_shell_args_vec = if let Some(s_arg_str) = shell_args_val.and_then(Value::as_str) {
			s_arg_str.split_whitespace().map(String::from).collect()
		} else if let Some(arr_val) = shell_args_val.and_then(Value::as_array) {
			arr_val.iter().filter_map(Value::as_str).map(String::from).collect()
		} else {
			Vec::new()
		};
		let cwd_opt_path = options_val.get("cwd").and_then(Value::as_str).map(PathBuf::from);
		let env_vars_opt_map = if let Some(env_map_val) = options_val.get("env").and_then(Value::as_object) {
			let mut env_map = HashMap::new();
			for (k, v_val) in env_map_val {
				if let Some(v_str) = v_val.as_str() {
					env_map.insert(k.clone(), v_str.to_string());
				} else if v_val.is_null() {
					warn!(
						"[TerminalState new] Ignoring null env var '{}'; unsetting not directly supported.",
						k
					);
				}
			}
			if env_map.is_empty() { None } else { Some(env_map) }
		} else {
			None
		};
		TerminalState {
			id,
			name,
			shell_path:final_shell_path,
			shell_args:final_shell_args_vec,
			cwd:cwd_opt_path,
			env:env_vars_opt_map,
			os_process_id:None,
			is_pty:options_val.get("isPty").and_then(Value::as_bool).unwrap_or(true),
			pty_input_tx:None,
			reader_task_handle:None,
			process_wait_handle:None,
		}
	}
}

/// Represents the state of an output channel.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OutputChannelState {
	pub name:String,
	pub language_id:Option<String>,
	pub buffer:String,
	pub visible:bool,
}

impl OutputChannelState {
	pub fn new(name:&str, language_id:Option<String>) -> Self {
		Self { name:name.to_string(), language_id, buffer:String::new(), visible:false }
	}
}

/// Represents a registered language feature provider from a sidecar.
/// Uses `CommonLangProviderType` from `Land_Common` and `ProviderOptionsDto`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderRegistration {
	pub handle:u32,
	pub provider_type:CommonLangProviderType,
	#[serde(rename = "selectorDto")] // To match original JSON if needed, or keep as selector
	pub selector: Value, // The DocumentSelector DTO (array of IDocumentFilterDto)
	pub sidecar_id:String, // ID of the sidecar that registered this provider
	#[serde(rename = "extensionIdDtoVal")] // To match original JSON if needed
	pub extension_id: Value, // Store the IExtensionIdentifier DTO from Cocoon

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "optionsDto")] // To match original JSON if needed
	pub options: Option<ProviderOptionsDto>, // Use the common DTO
}

/// Represents the metadata of a scanned extension, derived from its
/// `package.json`.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDescriptionState {
	pub identifier:Value, // Extension identifier DTO: `{ value: "publisher.name", uuid?: string }`
	pub name:String,
	pub version:String,
	pub publisher:String,
	pub engines:Value, // Engine compatibility, e.g., `{ "vscode": "^1.80.0" }`
	#[serde(skip_serializing_if = "Option::is_none")]
	pub main:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub browser:Option<String>,
	#[serde(rename = "type", skip_serializing_if = "Option::is_none")]
	pub module_type:Option<String>,
	#[serde(default)]
	pub is_builtin:bool,
	#[serde(default)]
	pub is_under_development:bool,
	pub extension_location:Value, // URI (as `UriComponents` DTO)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub activation_events:Option<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub contributes:Option<Value>,
}

/// Context for an active hierarchy session (Call Hierarchy, Type Hierarchy).
#[derive(Debug, Clone)]
pub struct HierarchySessionContext {
	pub original_provider_handle:u32,
	pub original_sidecar_id:String,
	pub provider_type:CommonLangProviderType,
}

// --- Central Application State Struct ---
#[derive(Clone)]
pub struct AppState {
	pub workspace_folders:Arc<StdMutex<Vec<WorkspaceFolderState>>>,
	pub workspace_config_path:Arc<StdMutex<Option<PathBuf>>>,
	pub is_trusted:Arc<AtomicBool>,
	pub configuration:Arc<StdMutex<MergedConfigurationState>>,
	pub global_memento:Arc<StdMutex<MementoStorageMap>>,
	pub global_memento_path:PathBuf,
	pub workspace_memento:Arc<StdMutex<MementoStorageMap>>,
	pub workspace_memento_path:Arc<StdMutex<Option<PathBuf>>>,
	pub command_registry:Arc<StdMutex<CommandRegistryMap>>,
	pub diagnostics_map:Arc<StdMutex<DiagnosticsStorageMap>>,
	pub open_documents:Arc<StdMutex<OpenDocumentMap>>,
	pub output_channels:Arc<StdMutex<OutputChannelStorageMap>>,
	pub language_providers:Arc<StdMutex<LanguageProviderRegistrationMap>>,
	pub next_provider_handle:Arc<AtomicU32>,
	pub scanned_extensions:Arc<StdMutex<ScannedExtensionMetadataMap>>,
	pub enabled_proposed_apis:Arc<StdMutex<EnabledProposedApisConfigMap>>,
	pub extension_scan_paths:Arc<StdMutex<Vec<PathBuf>>>,
	pub active_terminals:Arc<StdMutex<ActiveTerminalMap>>,
	pub next_terminal_id:Arc<AtomicU64>,
	pub pending_ui_requests:Arc<StdMutex<PendingUiRequestChannelMap>>,
	#[serde(skip)] // Don't persist this runtime state
	pub active_hierarchy_sessions: Arc<StdMutex<ActiveHierarchySessionMap>>,
}

// --- Helper Functions (module-private, used in AppState::default) ---
fn resolve_memento_storage_file_path(app_data_dir:&Path, is_global_scope:bool, workspace_id_str:&str) -> PathBuf {
	let user_storage_base_path = app_data_dir.join("User");
	if is_global_scope {
		user_storage_base_path.join("globalStorage.json")
	} else {
		let sanitized_workspace_id_segment =
			workspace_id_str.replace(|c:char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
		user_storage_base_path
			.join("workspaceStorage")
			.join(sanitized_workspace_id_segment)
			.join("storage.json")
	}
}

fn load_initial_memento_storage_from_disk(storage_file_path:&Path) -> MementoStorageMap {
	if !storage_file_path.exists() {
		debug!(
			"[AppState Init MementoLoad] Storage file not found: '{}'. Empty map.",
			storage_file_path.display()
		);
		return HashMap::new();
	}
	debug!(
		"[AppState Init MementoLoad] Loading memento from: {}",
		storage_file_path.display()
	);
	match fs::read_to_string(storage_file_path) {
		Ok(json_content_str) => {
			if json_content_str.trim().is_empty() {
				debug!(
					"[AppState Init MementoLoad] Storage file '{}' is empty. Empty map.",
					storage_file_path.display()
				);
				return HashMap::new();
			}
			match serde_json::from_str(&json_content_str) {
				Ok(parsed_map) => {
					info!(
						"[AppState Init MementoLoad] Loaded {} items from: {}",
						parsed_map.len(),
						storage_file_path.display()
					);
					parsed_map
				},
				Err(e_parse) => {
					error!(
						"[AppState Init MementoLoad] Failed to parse JSON from '{}': {}. Empty map.",
						storage_file_path.display(),
						e_parse
					);
					HashMap::new()
				},
			}
		},
		Err(e_read) => {
			if e_read.kind() != std::io::ErrorKind::NotFound {
				error!(
					"[AppState Init MementoLoad] Failed to read '{}': {}. Empty map.",
					storage_file_path.display(),
					e_read
				);
			} else {
				debug!(
					"[AppState Init MementoLoad] Storage file confirmed not found during read: {}",
					storage_file_path.display()
				);
			}
			HashMap::new()
		},
	}
}

pub fn analyze_text_lines_and_eol_for_document_state(text:&str) -> (Vec<String>, String) {
	let detected_eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
	let lines = text.split(detected_eol).map(String::from).collect();
	(lines, detected_eol.to_string())
}

// --- Default Implementation for AppState ---
impl Default for AppState {
	fn default() -> Self {
		info!("[AppState Default] Initializing default application state...");
		let app_name_for_paths = env!("CARGO_PKG_NAME");
		let app_data_dir_path = dirs::config_dir().map(|p| p.join(app_name_for_paths)).unwrap_or_else(|| {
			warn!(
				"[AppState Default] Could not get system config dir. Using relative path '.{}-appdata'.",
				app_name_for_paths
			);
			PathBuf::from(format!(".{}-appdata", app_name_for_paths))
		});
		if !app_data_dir_path.exists() {
			if let Err(e_create_dir) = fs::create_dir_all(&app_data_dir_path) {
				error!(
					"[AppState Default] CRITICAL: Failed to create app data dir at '{}': {}.",
					app_data_dir_path.display(),
					e_create_dir
				);
			}
		}
		let global_memento_file_path = resolve_memento_storage_file_path(&app_data_dir_path, true, "");
		debug!("[AppState Default] Global memento path: {}", global_memento_file_path.display());
		let initial_global_memento_map = load_initial_memento_storage_from_disk(&global_memento_file_path);
		let initial_workspace_memento_map = HashMap::new();
		let workspace_memento_file_path_arc_mutex = Arc::new(StdMutex::new(None));
		let mut initial_command_registry_map = HashMap::new();
		info!("[AppState Default] Registering native Mountain commands...");
		commands::register_native_command_internal(
			&mut initial_command_registry_map,
			"workbench.action.files.saveAll".to_string(),
			commands::handle_native_save_all::<Wry>,
		);
		commands::register_native_command_internal(
			&mut initial_command_registry_map,
			"mountain.action.showAbout".to_string(),
			commands::handle_native_show_about::<Wry>,
		);
		let scanned_extensions_map_arc_mutex = Arc::new(StdMutex::new(HashMap::new()));
		let enabled_proposed_apis_map_arc_mutex = Arc::new(StdMutex::new(HashMap::new()));
		let extension_scan_paths_arc_mutex = Arc::new(StdMutex::new(Vec::new()));
		info!(
			"[AppState Default] Default state init complete. App Data Dir: '{}'",
			app_data_dir_path.display()
		);
		AppState {
			workspace_folders:Arc::new(StdMutex::new(Vec::new())),
			configuration:Arc::new(StdMutex::new(MergedConfigurationState::default())),
			is_trusted:Arc::new(AtomicBool::new(false)),
			workspace_config_path:Arc::new(StdMutex::new(None)),
			command_registry:Arc::new(StdMutex::new(initial_command_registry_map)),
			diagnostics_map:Arc::new(StdMutex::new(HashMap::new())),
			open_documents:Arc::new(StdMutex::new(HashMap::new())),
			output_channels:Arc::new(StdMutex::new(HashMap::new())),
			global_memento:Arc::new(StdMutex::new(initial_global_memento_map)),
			global_memento_path:global_memento_file_path,
			workspace_memento:Arc::new(StdMutex::new(initial_workspace_memento_map)),
			workspace_memento_path:workspace_memento_file_path_arc_mutex,
			language_providers:Arc::new(StdMutex::new(HashMap::new())),
			next_provider_handle:Arc::new(AtomicU32::new(1)),
			scanned_extensions:scanned_extensions_map_arc_mutex,
			enabled_proposed_apis:enabled_proposed_apis_map_arc_mutex,
			extension_scan_paths:extension_scan_paths_arc_mutex,
			active_terminals:Arc::new(StdMutex::new(HashMap::new())),
			next_terminal_id:Arc::new(AtomicU64::new(1)),
			pending_ui_requests:Arc::new(StdMutex::new(HashMap::new())),
			active_hierarchy_sessions:Arc::new(StdMutex::new(HashMap::new())),
		}
	}
}

// --- AppState Methods ---
impl AppState {
	pub fn get_workspace_id_string(&self) -> Result<String, String> {
		let config_path_guard = self
			.workspace_config_path
			.lock()
			.map_err(|e| format!("Lock error on workspace_config_path: {}", e))?;
		if let Some(config_path) = config_path_guard.as_ref() {
			return Ok(config_path.file_name().unwrap_or_default().to_string_lossy().into_owned());
		}
		drop(config_path_guard);
		let folders_guard = self
			.workspace_folders
			.lock()
			.map_err(|e| format!("Lock error on workspace_folders: {}", e))?;
		if let Some(first_folder) = folders_guard.first() {
			return Ok(first_folder
				.uri
				.path()
				.replace(|c:char| !c.is_alphanumeric() && c != '/' && c != '\\', "_"));
		}
		Ok("NO_WORKSPACE".to_string())
	}

	pub fn update_workspace_memento_path_and_reload(&self, app_data_dir:&Path) -> Result<(), String> {
		let workspace_id_str = self.get_workspace_id_string()?;
		if workspace_id_str == "NO_WORKSPACE" {
			let mut path_guard = self
				.workspace_memento_path
				.lock()
				.map_err(|e| format!("Lock error (ws memento path for clear): {}", e))?;
			if path_guard.is_some() {
				info!("[AppState Memento] No active workspace, clearing workspace memento.");
				*path_guard = None;
				let mut memento_data_guard = self
					.workspace_memento
					.lock()
					.map_err(|e| format!("Lock error (ws memento data for clear): {}", e))?;
				memento_data_guard.clear();
			}
			return Ok(());
		}
		let new_memento_file_path = resolve_memento_storage_file_path(app_data_dir, false, &workspace_id_str);
		let mut path_guard = self
			.workspace_memento_path
			.lock()
			.map_err(|e| format!("Lock error (ws memento path for update): {}", e))?;
		if path_guard.as_ref() != Some(&new_memento_file_path) {
			info!(
				"[AppState Memento] Updating workspace memento path to: {}",
				new_memento_file_path.display()
			);
			if let Some(parent_dir) = new_memento_file_path.parent() {
				if !parent_dir.exists() {
					if let Err(e_create) = fs::create_dir_all(parent_dir) {
						error!(
							"[AppState Memento] Failed to create dir for ws memento at '{}': {}.",
							parent_dir.display(),
							e_create
						);
					}
				}
			}
			*path_guard = Some(new_memento_file_path.clone());
			debug!(
				"[AppState Memento] Reloading workspace memento from new path: {}",
				new_memento_file_path.display()
			);
			let new_memento_content_map = load_initial_memento_storage_from_disk(&new_memento_file_path);
			let mut memento_data_guard = self
				.workspace_memento
				.lock()
				.map_err(|e| format!("Lock error (ws memento data for reload): {}", e))?;
			*memento_data_guard = new_memento_content_map;
		}
		Ok(())
	}

	pub fn get_workspace_name(&self) -> Result<String, String> {
		let config_path_guard = self
			.workspace_config_path
			.lock()
			.map_err(|e| format!("Lock error (config path for ws name): {}", e))?;
		Ok(match config_path_guard.as_ref().and_then(|p| p.file_stem()) {
			Some(stem) => stem.to_string_lossy().into_owned(),
			None => {
				drop(config_path_guard);
				let folders_guard = self
					.workspace_folders
					.lock()
					.map_err(|e| format!("Lock error (folders for ws name): {}", e))?;
				match folders_guard.first() {
					Some(folder) => folder.name.clone(),
					None => "Untitled Workspace".to_string(),
				}
			},
		})
	}

	pub fn get_next_provider_handle(&self) -> u32 { self.next_provider_handle.fetch_add(1, AtomicOrdering::Relaxed) }

	pub async fn scan_extensions_and_populate_state(&self) {
		let current_scan_paths_vec = { self.extension_scan_paths.lock().unwrap_or_else(|e| e.into_inner()).clone() };
		info!("[AppState ExtScan] Starting scan in paths: {:?}", current_scan_paths_vec);
		let mut found_extensions_map = HashMap::new();
		for scan_dir_path in current_scan_paths_vec {
			if !scan_dir_path.is_dir() {
				warn!(
					"[AppState ExtScan] Scan path not a dir or DNE: '{}'. Skipping.",
					scan_dir_path.display()
				);
				continue;
			}
			match fs::read_dir(&scan_dir_path) {
				Ok(dir_entries) => {
					for entry_result in dir_entries {
						if let Ok(dir_entry) = entry_result {
							let extension_candidate_path = dir_entry.path();
							if extension_candidate_path.is_dir() {
								let package_json_file_path = extension_candidate_path.join("package.json");
								if package_json_file_path.is_file() {
									match fs::read_to_string(&package_json_file_path) {
										Ok(pkg_json_content_str) => {
											match serde_json::from_str::<Value>(&pkg_json_content_str) {
												Ok(pkg_json_val) => {
													if let (
														Some(name_val),
														Some(publisher_val),
														Some(version_val),
														Some(engines_val),
													) = (
														pkg_json_val.get("name"),
														pkg_json_val.get("publisher"),
														pkg_json_val.get("version"),
														pkg_json_val.get("engines"),
													) {
														if let (
															Some(name_str),
															Some(publisher_str),
															Some(version_str),
															Some(engines_vscode_str),
														) = (
															name_val.as_str(),
															publisher_val.as_str(),
															version_val.as_str(),
															engines_val.get("vscode").and_then(Value::as_str),
														) {
															let ext_id_str = format!("{}.{}", publisher_str, name_str);
															let ext_location_url =
																Url::from_directory_path(&extension_candidate_path)
																	.unwrap_or_else(|_| {
																		Url::parse(&format!(
																			"file:///{}",
																			extension_candidate_path
																				.to_string_lossy()
																				.replace('\\', "/")
																		))
																		.expect("Fallback URL parse failed")
																	});
															let ext_desc_state = ExtensionDescriptionState {
																identifier:json!({ "value": ext_id_str, "uuid": pkg_json_val.get("uuid").and_then(Value::as_str) }),
																name:name_str.to_string(),
																version:version_str.to_string(),
																publisher:publisher_str.to_string(),
																engines:json!({ "vscode": engines_vscode_str.to_string() }),
																main:pkg_json_val
																	.get("main")
																	.and_then(Value::as_str)
																	.map(String::from),
																browser:pkg_json_val
																	.get("browser")
																	.and_then(Value::as_str)
																	.map(String::from),
																module_type:pkg_json_val
																	.get("type")
																	.and_then(Value::as_str)
																	.map(String::from),
																is_builtin:true,
																is_under_development:false, /* TODO: Determine
																                             * accurately */
																extension_location:json!({"scheme": ext_location_url.scheme(), "authority": ext_location_url.host_str().unwrap_or(""), "path": ext_location_url.path(), "external": ext_location_url.to_string(), "$mid": 1}),
																activation_events:pkg_json_val
																	.get("activationEvents")
																	.and_then(Value::as_array)
																	.map(|arr| {
																		arr.iter()
																			.filter_map(|v| {
																				v.as_str().map(String::from)
																			})
																			.collect()
																	}),
																contributes:pkg_json_val.get("contributes").cloned(),
															};
															info!(
																"[AppState ExtScan] Scanned extension: {}",
																ext_id_str
															);
															found_extensions_map.insert(ext_id_str, ext_desc_state);
														} else {
															warn!(
																"[AppState ExtScan] Invalid package.json in '{}': \
																 missing core string fields.",
																package_json_file_path.display()
															);
														}
													} else {
														warn!(
															"[AppState ExtScan] Invalid package.json in '{}': core \
															 fields not found.",
															package_json_file_path.display()
														);
													}
												},
												Err(e_json_parse) => {
													warn!(
														"[AppState ExtScan] Failed to parse package.json from '{}': \
														 {}. Skipping.",
														package_json_file_path.display(),
														e_json_parse
													);
												},
											}
										},
										Err(e_read_file) => {
											warn!(
												"[AppState ExtScan] Failed to read package.json '{}': {}. Skipping.",
												package_json_file_path.display(),
												e_read_file
											);
										},
									}
								}
							}
						}
					}
				},
				Err(e_read_dir) => {
					error!(
						"[AppState ExtScan] Failed to read entries in scan path '{}': {}. Skipping.",
						scan_dir_path.display(),
						e_read_dir
					);
				},
			}
		}
		if !found_extensions_map.is_empty() {
			let mut scanned_extensions_guard = self.scanned_extensions.lock().unwrap_or_else(|e| e.into_inner());
			*scanned_extensions_guard = found_extensions_map;
			info!(
				"[AppState ExtScan] Extension scan complete. Total: {}",
				scanned_extensions_guard.len()
			);
		} else {
			info!("[AppState ExtScan] No extensions found.");
		}
	}

	pub fn get_next_terminal_id(&self) -> u64 { self.next_terminal_id.fetch_add(1, AtomicOrdering::Relaxed) }
}

// --- Serde Helper for serializing/deserializing url::Url ---
mod url_serde_helper {
	use serde::{self, Deserialize, Deserializer, Serializer};
	use url::Url;
	pub fn serialize<S>(url:&Url, serializer:S) -> Result<S::Ok, S::Error>
	where
		S: Serializer, {
		serializer.serialize_str(url.as_str())
	}
	pub fn deserialize<'de, D>(deserializer:D) -> Result<Url, D::Error>
	where
		D: Deserializer<'de>, {
		let s = String::deserialize(deserializer)?;
		Url::parse(&s).map_err(serde::de::Error::custom)
	}
}
