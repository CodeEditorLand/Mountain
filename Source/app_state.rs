// ---------------------------------------------------------------------------------------------
// Mountain Application State (app_state.rs)
// --------------------------------------------------------------------------------------------
// Defines the central `AppState` struct managed by Tauri via `app.manage()`.
// This struct aggregates all shared, mutable application state required across
// different parts of Mountain, including command handlers, effect
// implementations (Environment), and background tasks. State is wrapped
// appropriately (e.g., `Arc<Mutex>`, `Arc<AtomicBool>`) for thread-safe access.
//
// Responsibilities:
// - Defining the structure for shared state data:
//   - Workspace information (`workspace_folders`, `workspace_config_path`,
//     `is_trusted`).
//   - Merged configuration (`configuration`).
//   - Extension storage (`global_memento`, `workspace_memento`) and their
//     paths.
//   - Command registry (`command_registry`) storing native and proxied
//     handlers.
//   - Diagnostics store (`diagnostics_map`).
//   - Open document state (`open_documents`).
//   - Output channel state (`output_channels`).
//   - Language feature provider registrations (`language_providers`,
//     `next_provider_handle`).
//   - Extension descriptions (`scanned_extensions`) for populating initData.
//   - Proposed API configurations (`enabled_proposed_apis`).
//   - Paths for scanning extensions (`extension_scan_paths`).
//   - Terminal states (`active_terminals`, `next_terminal_id`).
//   - Pending UI request channels (`pending_ui_requests`).
// - Providing a `Default` implementation that initializes the state,
//   potentially loads persisted data from disk (e.g., mementos), and registers
//   native command handlers.
//
// Key Interactions:
// - Instantiated once and managed by Tauri (`app.manage(AppState::default())`).
// - Accessed via `app_handle.state::<AppState>()` in Tauri commands and setup
//   hooks.
// - Accessed via `self.get_app_state()` within the `MountainEnvironment`.
// - Read from and written to (under locks) by various handlers (`handlers/*`)
//   and effect implementations (`environment.rs`).
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// Used for synchronous I/O during initialization and current scan_extensions
	fs,

	path::{Path, PathBuf},

	sync::{
		Arc,

		// Standard Mutex
		Mutex as StdMutex,

		MutexGuard,

		atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
	},
};

use Land_Common::errors::CommonError;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
// Use Wry runtime as default
use tauri::{AppHandle, Manager, Runtime, Wry};
use tokio::sync::{
	// For JoinHandle wrappers in TerminalState
	Mutex as TokioMutex,

	// For terminal input channel sender if stored in TerminalState
	mpsc as TokioMpsc,

	// For pending UI requests
	oneshot,
};
// For terminal task handles
use tokio::task::JoinHandle;
use url::Url;

use crate::{
	handlers::{
		commands::{
			CommandHandler,

			handle_native_save_all,

			handle_native_show_about,

			register_native_command_internal,
		},

		// Struct for diagnostics data (from diagnostics.rs)
		diagnostics::MarkerData,
	},

	// Needed for native command handler signature
	runtime::AppRuntime,
	// For the Result type in PendingUiRequestMap
};

// --- Type Aliases ---
pub type CommandRegistry = HashMap<String, CommandHandler<Wry>>;

pub type DiagnosticsMap = HashMap<String /* owner */, HashMap<String /* UriString */, Vec<MarkerData>>>;

pub type StorageMap = HashMap<String /* key */, Value /* value */>;

pub type DocumentMap = HashMap<String /* UriString */, DocumentState>;

pub type OutputChannelMap = HashMap<String /* Channel ID */, OutputChannelState>;

pub type LanguageProviderMap = HashMap<u32 /* Handle */, ProviderRegistration>;

pub type ScannedExtensionMap = HashMap<String /* Extension ID (publisher.name) */, ExtensionDescriptionState>;

pub type EnabledProposedApisMap = HashMap<String /* Extension ID or '*' */, Vec<String /* proposal name */>>;

// Terminal ID (u64) to TerminalState
pub type TerminalMap = HashMap<u64, Arc<StdMutex<TerminalState>>>;

// Request ID to oneshot sender
pub type PendingUiRequestMap = HashMap<String, oneshot::Sender<Result<Value, CommonError>>>;

// --- State Structures ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkspaceFolderState {
	#[serde(with = "url_serde")]
	pub uri:Url,

	pub name:String,

	pub index:usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigurationState {
	/// Holds the merged configuration values.
	/// In a full implementation, this would be a more complex structure
	/// that understands configuration scopes (User, Workspace, Folder,
	///
	///
	/// Language) and can apply overrides. For MVP, it's a single JSON Value.
	pub data:Value,
}

impl ConfigurationState {
	pub fn new(data:Value) -> Self { Self { data } }

	/// Gets a configuration value.
	/// `section`: dot-separated path (e.g., "editor.fontSize").
	/// `_scope`: Currently unused placeholder for resource URI or language ID
	/// for scope-specific values.
	pub fn get_value(&self, section:Option<&str>, _scope:Option<&Value>) -> Value {
		trace!("[AppState Config] get_value: section={:?}, scope={:?}", section, _scope);

		if let Some(s) = section {
			let mut current = &self.data;

			for part in s.split('.') {
				if let Some(next) = current.get(part) {
					current = next;
				} else {
					trace!(
						"[AppState Config] Section part '{}' not found in config for section '{}'",
						part, s
					);

					// Not found
					return Value::Null;
				}
			}
			current.clone()
		} else {
			// Return all if no section
			self.data.clone()
		}
	}

	/// Updates the entire configuration state from a new state object.
	pub fn update_from(&mut self, new_state:ConfigurationState) {
		info!("[AppState Config] Updating entire configuration state.");

		trace!(
			"[AppState Config] Old data items: {}, New data items: {}",
			self.data.as_object().map_or(0, |o| o.len()),
			new_state.data.as_object().map_or(0, |o| o.len())
		);

		self.data = new_state.data;
	}
}

// For deserializing RpcModelContentChange within DocumentState::apply_changes
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RpcModelContentChange {
	range:RpcRange,

	// Not directly used in simple line-based Vec<String> model
	// range_offset: u32,

	// Not directly used
	// range_length: u32,
	text:String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RpcRange {
	// These are 0-indexed from VS Code (as per Cocoon's DTO contract)
	start_line_number:usize,

	start_column:usize,

	end_line_number:usize,

	end_column:usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentState {
	#[serde(with = "url_serde")]
	pub uri:Url,

	pub language_id:String,

	// Corresponds to VS Code's TextDocument.version
	pub version:i64,

	pub lines:Vec<String>,

	// End Of Line sequence ("\n" or "\r\n")
	pub eol:String,

	pub is_dirty:bool,

	// e.g., "utf8"
	pub encoding:String,
}

impl DocumentState {
	pub fn get_text(&self) -> String { self.lines.join(&self.eol) }

	/// Applies changes to the document's content.
	/// `new_version_id`: The version ID from the sidecar after the change.
	/// `changes_val`: A `serde_json::Value` representing an array of
	/// `RpcModelContentChange` DTOs.
	pub fn apply_changes(&mut self, new_version_id:i64, changes_val:&Value) -> Result<(), String> {
		if new_version_id <= self.version {
			warn!(
				"[DocState ApplyChanges] Ignoring stale changes (Incoming V{} <= Current V{}) for {}",
				new_version_id, self.version, self.uri
			);

			return Ok(());
		}

		debug!(
			"[DocState ApplyChanges] Applying V{} (Current V{}) for {}. Incoming changes: {:?}",
			new_version_id, self.version, self.uri, changes_val
		);

		// Attempt to deserialize into a Vec of RpcModelContentChange
		// VS Code sends changes in an order that they can be applied sequentially.
		let rpc_changes:Vec<RpcModelContentChange> = match serde_json::from_value(changes_val.clone()) {
			Ok(c) => c,

			Err(e) => {
				// Fallback: if it's just a string, treat as full text replacement
				if let Some(full_text) = changes_val.as_str() {
					info!(
						"[DocState ApplyChanges] Received full text string for V{}. Replacing content of {}.",
						new_version_id, self.uri
					);

					let (new_lines, new_eol) = lines_and_eol_from_text(full_text);

					self.lines = new_lines;

					// Assume sidecar provides correct EOL if full text
					self.eol = new_eol;

					self.version = new_version_id;

					self.is_dirty = true;

					return Ok(());
				}
				// If not a string and not a valid array of changes, it might be just a version
				// bump if the array is explicitly empty.
				if changes_val.as_array().map_or(true, |arr| arr.is_empty()) && new_version_id > self.version {
					debug!(
						"[DocState ApplyChanges] Applying version bump (V{} -> V{}) with no content changes for {}.",
						self.version, new_version_id, self.uri
					);

					self.version = new_version_id;

					// is_dirty might be set by a separate notification ($acceptDirtyStateChanged)
					return Ok(());
				}
				return Err(format!("Invalid RpcModelContentChange structure for {}: {}", self.uri, e));
			},
		};

		if rpc_changes.is_empty() && new_version_id > self.version {
			debug!(
				"[DocState ApplyChanges] Version bump (V{} -> V{}) with empty changes array for {}.",
				self.version, new_version_id, self.uri
			);

			self.version = new_version_id;

			return Ok(());
		}

		// Apply changes sequentially as they are sent by VS Code's model.
		for change in rpc_changes {
			// Convert 0-indexed DTO line/col to 0-indexed Vec/String indices for Rust.
			let start_line_idx = change.range.start_line_number;

			let mut start_col_idx = change.range.start_column;

			let end_line_idx = change.range.end_line_number;

			let mut end_col_idx = change.range.end_column;

			trace!(
				"[DocState ApplyChanges] Applying change: range L{}:C{} - L{}:C{}, text: '{}...'",
				// Log as 1-based for human readability
				start_line_idx + 1,
				start_col_idx + 1,
				end_line_idx + 1,
				end_col_idx + 1,
				change.text.chars().take(20).collect::<String>()
			);

			// Boundary checks for lines
			if start_line_idx > self.lines.len()
				|| end_line_idx > self.lines.len()
				|| (start_line_idx == self.lines.len() && start_col_idx > 0)
			{
				error!(
					"[DocState ApplyChanges] Invalid change range for {}: range L{}-L{} exceeds line count {}. \
					 Change: {:?}",
					self.uri,
					start_line_idx + 1,
					end_line_idx + 1,
					self.lines.len(),
					change
				);

				// Skip this invalid change
				continue;
			}
			// Clamp column indices to be within the bounds of their respective lines (char
			// counts)
			if start_line_idx < self.lines.len() {
				start_col_idx = std::cmp::min(start_col_idx, self.lines[start_line_idx].chars().count());
			} else if start_line_idx == self.lines.len() && start_col_idx != 0 {
				// Appending to new line after last line
				error!(
					"[DocState ApplyChanges] Invalid start column for append new line for {}: L{} C{}. Line count {}. \
					 Change: {:?}",
					self.uri,
					start_line_idx + 1,
					start_col_idx + 1,
					self.lines.len(),
					change
				);

				continue;
			}

			if end_line_idx < self.lines.len() {
				end_col_idx = std::cmp::min(end_col_idx, self.lines[end_line_idx].chars().count());
			} else if end_line_idx == self.lines.len() && end_col_idx != 0 {
				// Range ends on a new line after last line
				error!(
					"[DocState ApplyChanges] Invalid end column for append new line for {}: L{} C{}. Line count {}. \
					 Change: {:?}",
					self.uri,
					end_line_idx + 1,
					end_col_idx + 1,
					self.lines.len(),
					change
				);

				continue;
			}

			let text_to_insert_lines:Vec<String> = change.text.split(&self.eol).map(String::from).collect();

			if start_line_idx == end_line_idx {
				// Single-line change
				if start_line_idx >= self.lines.len() {
					// Adding new line(s) at the end
					if start_line_idx == self.lines.len() && start_col_idx == 0 && end_col_idx == 0 {
						self.lines.extend(text_to_insert_lines);
					} else {
						error!(
							"[DocState ApplyChanges] Attempting single-line change on non-existent line {} or invalid \
							 columns for {}. Change: {:?}",
							start_line_idx + 1,
							self.uri,
							change
						);

						continue;
					}
				} else {
					// Modify existing line
					let line = &mut self.lines[start_line_idx];

					let original_line_tail = line.chars().skip(end_col_idx).collect::<String>();

					let mut new_line_content = line.chars().take(start_col_idx).collect::<String>();

					if text_to_insert_lines.len() == 1 {
						// Inserted text is single line
						new_line_content.push_str(&text_to_insert_lines[0]);

						new_line_content.push_str(&original_line_tail);

						*line = new_line_content;
					} else {
						// Inserted text is multi-line, splitting the current line
						new_line_content.push_str(&text_to_insert_lines[0]);

						// Update the first part of the split line
						*line = new_line_content;

						// Insert the intermediate new lines
						for i in 1..text_to_insert_lines.len() - 1 {
							self.lines.insert(start_line_idx + i, text_to_insert_lines[i].clone());
						}

						// Add the last line of the inserted text, followed by the original line's tail
						let last_inserted_line_part = text_to_insert_lines.last().unwrap().clone();

						self.lines.insert(
							start_line_idx + text_to_insert_lines.len() - 1,
							last_inserted_line_part + &original_line_tail,
						);
					}
				}
			} else {
				// Multi-line change (delete range and insert new text)
				if start_line_idx >= self.lines.len() {
					error!(
						"[DocState ApplyChanges] Attempting multi-line change starting on non-existent line {} for \
						 {}. Change: {:?}",
						start_line_idx + 1,
						self.uri,
						change
					);

					continue;
				}

				// Preserve the part of the start line before the change
				let first_line_prefix = self.lines[start_line_idx].chars().take(start_col_idx).collect::<String>();

				// Preserve the part of the end line after the change
				let last_line_suffix = if end_line_idx < self.lines.len() {
					self.lines[end_line_idx].chars().skip(end_col_idx).collect::<String>()
				} else {
					// Deleting through the end of the document or up to a non-existent line
					String::new()
				};

				// Construct the new content for the start line
				let mut modified_start_line = first_line_prefix;

				modified_start_line.push_str(&text_to_insert_lines[0]);

				// If inserted text is single line, combine it with last_line_suffix on the
				// start_line_idx
				if text_to_insert_lines.len() == 1 {
					modified_start_line.push_str(&last_line_suffix);

					self.lines[start_line_idx] = modified_start_line;
				} else {
					// Inserted text is multi-line
					// Set the first modified line
					self.lines[start_line_idx] = modified_start_line;

					// Insert intermediate lines from text_to_insert_lines
					for i in 1..text_to_insert_lines.len() - 1 {
						self.lines.insert(start_line_idx + i, text_to_insert_lines[i].clone());
					}

					// Insert the last line of text_to_insert_lines, combined with last_line_suffix
					let final_inserted_line_content = text_to_insert_lines.last().unwrap().clone() + &last_line_suffix;

					self.lines
						.insert(start_line_idx + text_to_insert_lines.len() - 1, final_inserted_line_content);
				}

				// Remove the original lines that were spanned by the multi-line range,

				// accounting for lines possibly added/removed by the insertion.
				// Original lines from (start_line_idx + 1) up to end_line_idx (inclusive) need
				// to be removed. The indices for removal are relative to the state *after*
				// insertions above the removal point.

				// Number of lines in the original range (excluding the start line, including
				// the end line)
				let num_original_lines_in_deleted_range_after_start = end_line_idx - start_line_idx;

				if num_original_lines_in_deleted_range_after_start > 0 {
					// The removal should start after all newly inserted lines (or after the
					// modified start line if insertion was 1 line)
					let removal_start_actual_idx = start_line_idx + std::cmp::max(1, text_to_insert_lines.len());

					if removal_start_actual_idx < self.lines.len() {
						// Ensure removal_start is within bounds
						let removal_end_actual_idx = std::cmp::min(
							self.lines.len(),
							removal_start_actual_idx + num_original_lines_in_deleted_range_after_start,
						);

						if removal_start_actual_idx < removal_end_actual_idx {
							self.lines.drain(removal_start_actual_idx..removal_end_actual_idx);
						}
					} else if removal_start_actual_idx == self.lines.len()
						&& num_original_lines_in_deleted_range_after_start > 0
					{

						// This means we inserted lines, and the original range
						// to delete might now be entirely beyond the
						// current document end or just at the end.
						// If we are deleting lines that effectively don't exist
						// after insertion, this is fine.
					} else if num_original_lines_in_deleted_range_after_start > 0 {
						// If removal_start_actual_idx > self.lines.len()
						debug!(
							"[DocState ApplyChanges] Calculated removal start index {} is out of bounds (lines: {}). \
							 No lines drained. URI: {}",
							removal_start_actual_idx,
							self.lines.len(),
							self.uri
						);
					}
				}
			}
		}

		self.version = new_version_id;

		// Any change implies dirty
		self.is_dirty = true;

		Ok(())
	}
}

// Removed Default as some fields are not easily defaultable (e.g. task handles)
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

	// Channel to send input to the PTY writer task
	// This sender can be cloned by `handle_sendText`.
	#[serde(skip)]
	pub pty_input_tx:Option<TokioMpsc::Sender<String>>,

	// Join handles for managing the spawned tasks.
	// Wrapped in Arc<TokioMutex<Option<...>>> to allow tasks to be taken/aborted once.
	#[serde(skip)]
	pub reader_task_handle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,

	#[serde(skip)]
	pub process_wait_handle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
	// DEV_NOTE: The PTY writer task JoinHandle could also be stored if direct cancellation is needed,

	// though it usually exits when pty_input_tx is dropped or the PTY master handle is closed.
}

impl TerminalState {
	pub fn new(id:u64, name:String, options:&Value, default_shell:String) -> Self {
		let shell_path = options
			.get("shellPath")
			.and_then(Value::as_str)
			.map(String::from)
			.unwrap_or(default_shell);

		let shell_args_val = options.get("shellArgs");

		let shell_args = if let Some(s_val) = shell_args_val.and_then(Value::as_str) {
			vec![s_val.to_string()]
		} else if let Some(arr_val) = shell_args_val.and_then(Value::as_array) {
			arr_val.iter().filter_map(Value::as_str).map(String::from).collect()
		} else {
			Vec::new()
		};

		let cwd = options.get("cwd").and_then(Value::as_str).map(PathBuf::from);

		let env_vars = if let Some(env_map_val) = options.get("env").and_then(Value::as_object) {
			let mut env_map = HashMap::new();

			for (k, v_val) in env_map_val {
				if let Some(v_str) = v_val.as_str() {
					env_map.insert(k.clone(), v_str.to_string());
				} else if v_val.is_null() {
					// DEV_NOTE: Unsetting environment variables via `null` in the `env` map option
					// is not directly supported by `CommandBuilder` if inheriting the parent
					// environment. It typically only allows adding or overriding. If not
					// inheriting, one can construct a completely new environment. For now,

					// `null` values are ignored.
					warn!(
						"[TerminalState new] Ignoring null value for env var '{}'; unsetting not directly supported.",
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

			shell_path,

			shell_args,

			cwd,

			env:env_vars,

			os_process_id:None,

			is_pty:options.get("isPty").and_then(Value::as_bool).unwrap_or(true),

			// Will be set after PTY and writer task are created
			pty_input_tx:None,

			// Will be set after reader task is spawned
			reader_task_handle:None,

			// Will be set after process wait task is spawned
			process_wait_handle:None,
		}
	}
}

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LanguageProviderType {
	Hover,

	Completion,

	Definition,

	Declaration,

	Implementation,

	TypeDefinition,

	References,

	DocumentHighlight,

	DocumentSymbol,

	WorkspaceSymbol,

	CodeAction,

	CodeLens,

	// General Formatting (covers DocumentFormattingEditProvider)
	Formatting,

	// Covers DocumentRangeFormattingEditProvider
	RangeFormatting,

	OnTypeFormatting,

	Rename,

	DocumentLink,

	// Covers DocumentColorProvider
	Color,

	FoldingRange,

	SelectionRange,

	CallHierarchy,

	TypeHierarchy,

	LinkedEditingRange,

	InlayHints,
	// TODO (Feature): Add SignatureHelp if its metadata (SignatureHelpProviderMetadataDto) is distinct enough
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderRegistration {
	// Unique handle generated by Mountain
	pub handle:u32,

	pub provider_type:LanguageProviderType,

	// The DocumentFilter JSON Value
	pub selector:Value,

	// ID of the sidecar that registered this
	pub sidecar_id:String,

	// Optional metadata, specific to provider types
	#[serde(skip_serializing_if = "Option::is_none")]
	// For Completion, SignatureHelp
	pub trigger_characters: Option<Vec<String>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// For CompletionItemProvider, InlayHintProvider
	pub supports_resolve_details: Option<bool>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// For CodeActionProvider (e.g., CodeActionProviderMetadataDto)
	pub code_action_metadata: Option<Value>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// For SignatureHelpProvider (e.g., SignatureHelpProviderMetadataDto)
	pub signature_help_metadata: Option<Value>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDescriptionState {
	// { value: string, uuid?: string }
	pub identifier:Value,

	pub name:String,

	pub version:String,

	pub publisher:String,

	// { vscode: string }
	pub engines:Value,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Entry point for Node.js
	pub main: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Entry point for Web
	pub browser: Option<String>,

	#[serde(rename = "type", skip_serializing_if = "Option::is_none")]
	// "commonjs" or "module" (for ESM)
	pub module_type: Option<String>,

	#[serde(default)]
	pub is_builtin:bool,

	#[serde(default)]
	pub is_under_development:bool,

	// UriComponents { scheme, path, authority, external, ... }
	pub extension_location:Value,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub activation_events:Option<Vec<String>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// The whole 'contributes' object
	pub contributes: Option<Value>,
}

// --- Central Application State ---
#[derive(Clone)]
pub struct AppState {
	pub workspace_folders:Arc<StdMutex<Vec<WorkspaceFolderState>>>,

	pub configuration:Arc<StdMutex<ConfigurationState>>,

	pub is_trusted:Arc<AtomicBool>,

	// Path to .code-workspace file or None
	pub workspace_config_path:Arc<StdMutex<Option<PathBuf>>>,

	pub command_registry:Arc<StdMutex<CommandRegistry>>,

	pub diagnostics_map:Arc<StdMutex<DiagnosticsMap>>,

	pub open_documents:Arc<StdMutex<DocumentMap>>,

	pub output_channels:Arc<StdMutex<OutputChannelMap>>,

	pub global_memento:Arc<StdMutex<StorageMap>>,

	// Resolved path to globalStorage.json
	pub global_memento_path:PathBuf,

	pub workspace_memento:Arc<StdMutex<StorageMap>>,

	// Resolved path, None if no workspace
	pub workspace_memento_path:Arc<StdMutex<Option<PathBuf>>>,

	pub language_providers:Arc<StdMutex<LanguageProviderMap>>,

	// Counter for generating unique handles
	pub next_provider_handle:Arc<AtomicU32>,

	/// All extensions Mountain knows about (e.g., scanned from disk). Key is
	/// `publisher.name`.
	pub scanned_extensions:Arc<StdMutex<ScannedExtensionMap>>,

	/// Configuration for proposed APIs. Key: extensionId or `*`, Value: list of
	/// proposal names.
	pub enabled_proposed_apis:Arc<StdMutex<EnabledProposedApisMap>>,

	/// Paths to directories where pre-bundled/scanned extensions are located.
	/// Modified by `main.rs` setup logic.
	pub extension_scan_paths:Arc<StdMutex<Vec<PathBuf>>>,

	/// Active terminal instances.
	pub active_terminals:Arc<StdMutex<TerminalMap>>,

	/// Counter for generating unique terminal IDs.
	pub next_terminal_id:Arc<AtomicU64>,

	/// Stores `oneshot::Sender` channels for pending UI requests made from
	/// async Rust to the Tauri frontend (Sky), awaiting a response.
	pub pending_ui_requests:Arc<StdMutex<PendingUiRequestMap>>,
}

// --- Helper Functions (module-private) ---

/// Helper to determine the path for persistent extension storage (memento).
fn resolve_storage_path(app_data_dir:&Path, scope_is_global:bool, workspace_id_or_empty:&str) -> PathBuf {
	// VS Code uses 'User' under app data
	let storage_base = app_data_dir.join("User");

	if scope_is_global {
		storage_base.join("globalStorage.json")
	} else {
		// Sanitize workspace_id to make it a valid directory name
		let sanitized_ws_id = workspace_id_or_empty.replace(|c:char| !c.is_alphanumeric() && c != '-' && c != '_', "_");

		// VS Code uses storage.json per workspace
		storage_base.join("workspaceStorage").join(sanitized_ws_id).join("storage.json")
	}
}

/// Helper function to load initial storage data from a JSON file.
/// Uses blocking I/O, suitable only for synchronous initialization contexts.
fn load_initial_storage(path:&Path) -> StorageMap {
	if !path.exists() {
		debug!("[AppState Init] Storage file not found, creating empty map: {}", path.display());

		return HashMap::new();
	}
	debug!("[AppState Init] Attempting to load storage from: {}", path.display());

	match fs::read_to_string(path) {
		Ok(content) => {
			match serde_json::from_str(&content) {
				Ok(map) => {
					info!(
						"[AppState Init] Successfully loaded {} items from storage: {}",
						map.len(),
						path.display()
					);

					map
				},

				Err(e) => {
					error!(
						"[AppState Init] Failed to parse storage file {}, returning empty map: {}",
						path.display(),
						e
					);

					HashMap::new()
				},
			}
		},

		Err(e) => {
			if e.kind() != std::io::ErrorKind::NotFound {
				error!(
					"[AppState Init] Failed to read storage file {}, returning empty map: {}",
					path.display(),
					e
				);
			} else {
				debug!("[AppState Init] Storage file confirmed not found: {}", path.display());
			}
			HashMap::new()
		},
	}
}

/// Helper function to split text into lines and detect EOL, used by
/// DocumentState.
fn lines_and_eol_from_text(text:&str) -> (Vec<String>, String) {
	// Simplified EOL detection for this context
	let detected_eol = if text.contains("\r\n") { "\r\n" } else { "\n" };

	let lines = text.split(detected_eol).map(String::from).collect();

	(lines, detected_eol.to_string())
}

// --- Default Implementation for AppState ---
impl Default for AppState {
	/// Initializes the `AppState` with default values. Runs synchronously
	/// during Tauri setup.
	fn default() -> Self {
		// Keep: Indicates a major lifecycle event
		info!("[AppState] Initializing default state...");

		// Determine App Data Directory
		// TODO (Robustness): Consider making the application name ("LandCodeEditor",

		// "Mountain", etc.) configurable, possibly via build scripts or an
		// environment variable at runtime, instead of hardcoding CARGO_PKG_NAME.
		// Uses package name from Cargo.toml
		let app_name = env!("CARGO_PKG_NAME");

		let app_data_dir_opt = dirs::config_dir().map(|p| p.join(app_name));

		let app_data_dir = app_data_dir_opt.unwrap_or_else(|| {
			warn!(
				"[AppState Init] Could not determine system config/data directory. Using relative '.{}-data'.",
				app_name
			);

			PathBuf::from(format!(".{}-data", app_name))
		});

		if !app_data_dir.exists() {
			if let Err(e) = fs::create_dir_all(&app_data_dir) {
				error!(
					"[AppState Init] Failed to create app data directory {}: {}",
					app_data_dir.display(),
					e
				);
			}
		}

		// Keep: Useful for debugging persistence issues
		let global_memento_path = resolve_storage_path(&app_data_dir, true, "");

		debug!("[AppState Init] Global memento path: {}", global_memento_path.display());

		let initial_global_memento = load_initial_storage(&global_memento_path);

		// Workspace memento is initially empty and path is None until a workspace is
		// opened
		let initial_workspace_memento = HashMap::new();

		let workspace_memento_path = Arc::new(StdMutex::new(None));

		// Initialize Command Registry & Register Native Commands
		let mut initial_command_registry = HashMap::new();

		// Keep: Useful for verifying native commands
		info!("[AppState Init] Registering native commands...");

		register_native_command_internal(
			&mut initial_command_registry,
			"workbench.action.files.saveAll".to_string(),
			handle_native_save_all::<Wry>,
		);

		register_native_command_internal(
			&mut initial_command_registry,
			"mountain.action.showAbout".to_string(),
			handle_native_show_about::<Wry>,
		);

		// TODO (Feature): Add more native commands here as the application grows (e.g.,

		// file operations, settings UI).

		let scanned_extensions = Arc::new(StdMutex::new(HashMap::new()));

		let enabled_proposed_apis = Arc::new(StdMutex::new(HashMap::new()));

		// `extension_scan_paths` is initialized as empty. It will be populated later,

		// typically in `main.rs` setup after `AppHandle` is available to resolve paths.
		let extension_scan_paths = Arc::new(StdMutex::new(Vec::new()));

		info!(
			"[AppState] Default initialization complete. Global Memento Path: {}",
			global_memento_path.display()
		);

		AppState {
			workspace_folders:Arc::new(StdMutex::new(Vec::new())),

			configuration:Arc::new(StdMutex::new(ConfigurationState::default())),

			// Default to not trusted
			is_trusted:Arc::new(AtomicBool::new(false)),

			workspace_config_path:Arc::new(StdMutex::new(None)),

			command_registry:Arc::new(StdMutex::new(initial_command_registry)),

			diagnostics_map:Arc::new(StdMutex::new(HashMap::new())),

			open_documents:Arc::new(StdMutex::new(HashMap::new())),

			output_channels:Arc::new(StdMutex::new(HashMap::new())),

			global_memento:Arc::new(StdMutex::new(initial_global_memento)),

			global_memento_path,

			workspace_memento:Arc::new(StdMutex::new(initial_workspace_memento)),

			workspace_memento_path,

			language_providers:Arc::new(StdMutex::new(HashMap::new())),

			// Start handles at 1
			next_provider_handle:Arc::new(AtomicU32::new(1)),

			scanned_extensions,

			enabled_proposed_apis,

			extension_scan_paths,

			active_terminals:Arc::new(StdMutex::new(HashMap::new())),

			// Start terminal IDs at 1
			next_terminal_id:Arc::new(AtomicU64::new(1)),

			pending_ui_requests:Arc::new(StdMutex::new(HashMap::new())),
		}
	}
}

// --- AppState Methods ---
impl AppState {
	/// Helper method to determine a unique ID string for the current workspace.
	/// Used for scoping workspace-specific storage.
	pub fn get_workspace_id_string(&self) -> Result<String, String> {
		// Prefer .code-workspace file path for ID if available
		let config_path_guard = self
			.workspace_config_path
			.lock()
			.map_err(|e| format!("Lock error (config path): {}", e))?;

		if let Some(config_path) = config_path_guard.as_ref() {
			// Using the file name; a more robust hash of the full canonical path is better
			// for uniqueness.
			// DEV_NOTE: Consider using a SHA256 hash of the canonicalized config_path for a
			// more robust ID. e.g., sha256(config_path.canonicalize().
			// unwrap_or(config_path.clone()).to_string_lossy())
			return Ok(config_path.file_name().unwrap_or_default().to_string_lossy().into_owned());
		}
		// Release lock
		drop(config_path_guard);

		// If no .code-workspace, use the path of the first workspace folder
		let folders_guard = self
			.workspace_folders
			.lock()
			.map_err(|e| format!("Lock error (folders): {}", e))?;

		if let Some(first_folder) = folders_guard.first() {
			// Using sanitized URI path; a hash of the canonical URI path would be more
			// robust.
			return Ok(first_folder.uri.path().replace(|c:char| !c.is_alphanumeric(), "_"));
		}

		Ok("NO_WORKSPACE".to_string())
	}

	/// Updates the workspace memento path when a workspace is opened or its ID
	/// changes. This should be called after `workspace_folders` and
	/// `workspace_config_path` are set. `app_data_dir` should be the resolved
	/// application data directory.
	pub fn update_workspace_memento_path(&self, app_data_dir:&Path) -> Result<(), String> {
		let workspace_id_str = self.get_workspace_id_string()?;

		let new_path = resolve_storage_path(app_data_dir, false, &workspace_id_str);

		let mut path_guard = self
			.workspace_memento_path
			.lock()
			.map_err(|e| format!("Lock error (workspace memento path): {}", e))?;

		if path_guard.as_ref() != Some(&new_path) {
			info!("[AppState] Updating workspace memento path to: {}", new_path.display());

			if !new_path.parent().map_or(false, |p| p.exists()) {
				if let Err(e) = fs::create_dir_all(new_path.parent().unwrap()) {
					error!(
						"[AppState] Failed to create directory for workspace memento {}: {}",
						new_path.display(),
						e
					);

					// Proceed with setting path, load_initial_storage will
					// handle non-existent file
				}
			}
			*path_guard = Some(new_path.clone());

			// When path changes, reload the workspace memento content
			debug!("[AppState] Reloading workspace memento from new path: {}", new_path.display());

			let new_memento_content = load_initial_storage(&new_path);

			let mut memento_guard = self
				.workspace_memento
				.lock()
				.map_err(|e| format!("Lock error (workspace memento data): {}", e))?;

			*memento_guard = new_memento_content;
		}
		Ok(())
	}

	/// Helper method to determine the display name for the current workspace.
	pub fn get_workspace_name(&self) -> Result<String, String> {
		let path_guard = self
			.workspace_config_path
			.lock()
			.map_err(|e| format!("Lock error (config path): {}", e))?;

		Ok(match path_guard.as_ref().and_then(|p| p.file_stem()) {
			Some(stem) => stem.to_string_lossy().into_owned(),

			None => {
				// Release lock before acquiring another
				drop(path_guard);

				let folders_guard = self
					.workspace_folders
					.lock()
					.map_err(|e| format!("Lock error (folders): {}", e))?;

				match folders_guard.first() {
					Some(folder) => folder.name.clone(),

					None => "Untitled Workspace".to_string(),
				}
			},
		})
	}

	/// Generates the next unique handle for a language provider registration.
	pub fn get_next_provider_handle(&self) -> u32 { self.next_provider_handle.fetch_add(1, Ordering::Relaxed) }

	/// Scans configured extension paths for `package.json` files and populates
	/// `self.scanned_extensions`.
	/// TODO (Robustness): This method is `async fn` but currently uses
	/// synchronous `std::fs` calls. For true async behavior, refactor to use
	/// `tokio::fs` and stream processing. This might be called during startup
	/// where some blocking is permissible, but a fully async version would be
	/// preferable for responsiveness and to avoid blocking the tokio runtime.
	pub async fn scan_extensions(&self) {
		let current_scan_paths = {
			// Clone paths to avoid holding lock during IO
			let guard = self.extension_scan_paths.lock().unwrap_or_else(|e| {
				error!("[AppState scan_extensions] Poisoned lock on extension_scan_paths: {}", e);

				// Attempt to recover by taking the inner data
				e.into_inner()
			});

			guard.clone()
		};

		info!("[AppState] Scanning for extensions in paths: {:?}", current_scan_paths);

		let mut found_extensions = HashMap::new();

		for scan_path in current_scan_paths {
			// Iterate over cloned paths
			if !scan_path.is_dir() {
				warn!(
					"[AppState] Extension scan path is not a directory or does not exist: {}",
					scan_path.display()
				);

				continue;
			}

			// NOTE: Synchronous directory walk. See TODO above.
			match fs::read_dir(scan_path) {
				Ok(entries) => {
					for entry_res in entries {
						if let Ok(entry) = entry_res {
							let path = entry.path();

							if path.is_dir() {
								let package_json_path = path.join("package.json");

								if package_json_path.is_file() {
									match fs::read_to_string(&package_json_path) {
										Ok(content) => {
											match serde_json::from_str::<Value>(&content) {
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
															Some(name),
															Some(publisher),
															Some(version),
															Some(engines_vscode),
														) = (
															name_val.as_str(),
															publisher_val.as_str(),
															version_val.as_str(),
															engines_val.get("vscode").and_then(Value::as_str),
														) {
															let ext_id_str = format!("{}.{}", publisher, name);

															let ext_location_url = Url::from_directory_path(&path)
																.unwrap_or_else(|_| {
																	warn!(
																		"[AppState] Failed to create directory URL \
																		 for extension path: {}",
																		path.display()
																	);

																	// Fallback to a generic file URL, might not be
																	// ideal
																	Url::parse(&format!(
																		"file:///{}",
																		path.to_string_lossy().replace("\\", "/")
																	))
																	.unwrap_or_else(|_| {
																		Url::parse("file:///unknown_extension_path")
																			.unwrap()
																	})
																});

															let desc_state = ExtensionDescriptionState {
																identifier:json!({ "value": ext_id_str, "uuid": pkg_json_val.get("uuid").and_then(Value::as_str) }),

																name:name.to_string(),

																version:version.to_string(),

																publisher:publisher.to_string(),

																engines:json!({ "vscode": engines_vscode.to_string() }),

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

																is_builtin:true, /* MVP_LIMITATION: Assumption:
																                  * Scanned
																                  * extensions are
																                  * "built-in" or trusted by default.
																                  * A more robust system would
																                  * differentiate. */
																is_under_development:false, /* Default, could be
																                             * parsed if available in
																                             * package.json */
																extension_location:json!({


																	"scheme": ext_location_url.scheme(),


																	"authority": ext_location_url.host_str().unwrap_or(""),


																	"path": ext_location_url.path(),


																	"external": ext_location_url.to_string()
																}),

																activation_events:pkg_json_val
																	.get("activationEvents")
																	.and_then(|ae| ae.as_array())
																	.map(|arr| {
																		arr.iter()
																			.filter_map(|v| {
																				v.as_str().map(String::from)
																			})
																			.collect()
																	}),

																contributes:pkg_json_val.get("contributes").cloned(),
															};

															info!("[AppState] Scanned extension: {}", ext_id_str);

															found_extensions.insert(ext_id_str, desc_state);
														} else {
															warn!(
																"[AppState] Invalid package.json (missing \
																 name/publisher/version/engines.vscode string \
																 fields): {}",
																package_json_path.display()
															);
														}
													} else {
														warn!(
															"[AppState] Invalid package.json (core fields not found): \
															 {}",
															package_json_path.display()
														);
													}
												},

												Err(e) => {
													warn!(
														"[AppState] Failed to parse package.json {}: {}",
														package_json_path.display(),
														e
													)
												},
											}
										},

										Err(e) => {
											warn!(
												"[AppState] Failed to read package.json {}: {}",
												package_json_path.display(),
												e
											)
										},
									}
								}
							}
						}
					}
				},

				Err(e) => error!("[AppState] Failed to read extension scan path {}: {}", scan_path.display(), e),
			}
		}

		if !found_extensions.is_empty() {
			let mut scanned_extensions_guard = self.scanned_extensions.lock().unwrap_or_else(|e| {
				error!("[AppState scan_extensions] Poisoned lock on scanned_extensions: {}", e);

				e.into_inner()
			});

			*scanned_extensions_guard = found_extensions;

			info!(
				"[AppState] Updated scanned extensions. Count: {}",
				scanned_extensions_guard.len()
			);
		} else {
			info!("[AppState] No extensions found in configured scan paths.");
		}
	}

	/// Generates the next unique ID for a terminal instance.
	pub fn get_next_terminal_id(&self) -> u64 { self.next_terminal_id.fetch_add(1, Ordering::Relaxed) }

	// TODO (Feature): Add an async initialization method `async fn
	// initialize_from_app_handle(&self, app_handle: &AppHandle<Wry>)`
	//       to handle logic requiring AppHandle (e.g., resolving paths for
	// `extension_scan_paths`       from app config or data directories, loading
	// last session state, triggering initial       extension scan, etc.) after the
	// AppHandle is available during setup.
}

// --- Serde Helper for Url ---
// Keep this helper accessible, either here or in a shared utils module.
mod url_serde {

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
