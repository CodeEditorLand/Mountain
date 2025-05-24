// ---------------------------------------------------------------------------------------------
// Mountain Application State (app_state.rs)
// --------------------------------------------------------------------------------------------
// Defines the central `AppState` struct managed by Tauri via `app.manage()`.
// This struct aggregates all shared, mutable application state required across
// different parts of Mountain, including command handlers, effect
// implementations (e.g., `MountainEnvironment`), and background tasks. State is
// wrapped appropriately (e.g., `Arc<StdMutex<_>>`, `Arc<AtomicBool>`) for
// thread-safe access from both synchronous and asynchronous contexts.
//
// Responsibilities:
// - Defining the structure for all shared application state data, including:
//   - Workspace Information: `workspace_folders`, `workspace_config_path` (path
//     to `.code-workspace` file if any), and `is_trusted` flag.
//   - Merged Configuration: `configuration` (holds the effective settings from
//     user, workspace, and folder levels).
//   - Extension Storage (Mementos): `global_memento` and `workspace_memento`
//     HashMaps, along with their respective file paths for persistence
//     (`global_memento_path`, `workspace_memento_path`).
//   - Command Registry: `command_registry` storing both native Mountain command
//     handlers and proxied handlers for commands registered by sidecars.
//   - Diagnostics Store: `diagnostics_map` holding diagnostic markers reported
//     by various owners (e.g., language servers).
//   - Open Document State: `open_documents` map tracking the state of currently
//     open text documents.
//   - Output Channel State: `output_channels` map managing output channels
//     created by extensions.
//   - Language Feature Provider Registrations: `language_providers` map and
//     `next_provider_handle` counter for managing providers registered by
//     extensions (e.g., for hover, completion).
//   - Scanned Extension Descriptions: `scanned_extensions` map containing
//     metadata for all extensions Mountain is aware of (used for `initData` to
//     Cocoon).
//   - Proposed API Configurations: `enabled_proposed_apis` map defining which
//     experimental/proposed APIs are enabled for which extensions.
//   - Extension Scan Paths: `extension_scan_paths` list of directories where
//     Mountain looks for extensions.
//   - Active Terminal States: `active_terminals` map and `next_terminal_id`
//     counter for managing integrated terminal instances.
//   - Pending UI Requests: `pending_ui_requests` map storing `oneshot::Sender`
//     channels for UI interactions initiated by Mountain and awaiting response
//     from the Sky frontend.
// - Providing a `Default` implementation (`AppState::default()`) that:
//   - Initializes all state fields to their default values.
//   - Loads persisted data from disk where applicable (e.g., global memento).
//   - Registers native Mountain command handlers into the `command_registry`.
// - Providing helper methods for common state-related operations, such as:
//   - `get_workspace_id_string()`: Determines a unique ID for the current
//     workspace.
//   - `update_workspace_memento_path()`: Updates the path for workspace memento
//     and reloads its content.
//   - `get_workspace_name()`: Determines the display name for the current
//     workspace.
//   - `get_next_provider_handle()`: Atomically gets the next unique ID for
//     language providers.
//   - `scan_extensions()`: Scans configured paths for extension `package.json`
//     files and populates `scanned_extensions`.
//   - `get_next_terminal_id()`: Atomically gets the next unique ID for
//     terminals.
//
// Key Interactions:
// - Instantiated once using `AppState::default()` and managed by Tauri via
//   `app.manage(initial_app_state)`.
// - Accessed throughout the application (handlers, effects, tasks) via
//   `app_handle.state::<AppState>()` or `self.get_app_state()` (within
//   `MountainEnvironment`).
// - Its fields are read from and written to (under appropriate locks) by
//   various handler modules (`handlers/*`) and effect implementations in
//   `environment.rs`.
// - The `scan_extensions` method is typically called during application setup
//   after `AppHandle` is available to resolve resource paths.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// Used for synchronous I/O during initialization and current `scan_extensions`
	fs,

	path::{Path, PathBuf},

	sync::{
		Arc,

		// Standard Mutex for thread-safe interior mutability
		Mutex as StdMutex,

		atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering as AtomicOrdering},
	},
};

// For `PendingUiRequestMap`'s Result type
use Land_Common::errors::CommonError;
// Logging
use log::{debug, error, info, trace, warn};
// For serializing/deserializing state DTOs
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
// Tauri essentials
use tauri::{Manager, Wry};
use tokio::sync::{
	// Tokio Mutex for JoinHandle wrappers in TerminalState
	Mutex as TokioMutex,

	// Tokio MPSC for terminal input channel sender
	mpsc as TokioMpsc,

	// Tokio oneshot for pending UI requests
	oneshot as TokioOneshot,
};
// For storing handles to spawned terminal tasks
use tokio::task::JoinHandle;
// For URI handling in various state DTOs
use url::Url;

use crate::handlers::{
	commands::{
		// Make module directly accessible
		self,

		// Enum for native/proxied command handlers
		CommandHandler,
	},

	// DTO for diagnostic markers
	diagnostics::MarkerData,
};

// --- Type Aliases for Clarity ---
// Wry is the default Tauri runtime
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

// --- State Structures (DTOs for complex state fields) ---

/// Represents the state of a single workspace folder.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkspaceFolderState {
	// Custom serializer for url::Url
	#[serde(with = "url_serde_helper")]
	// URI of the folder (e.g., "file:///path/to/folder")
	pub uri: Url,

	// Display name of the folder
	pub name:String,

	// Order/index of the folder within the workspace
	pub index:usize,
}

/// Represents the merged configuration state of the application.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MergedConfigurationState {
	/// Holds the effective, merged configuration values from all sources (user,
	///
	///
	/// workspace, folder). In a more complex system, this might be a
	/// structured representation that understands scopes and overrides. For
	/// MVP, it's a single `serde_json::Value` object.
	pub data:Value,
}

impl MergedConfigurationState {
	/// Creates a new `MergedConfigurationState` with the given data.
	pub fn new(data:Value) -> Self { Self { data } }

	/// Gets a configuration value for a given section (dot-separated path).
	///
	/// `_scope_uri_components` is currently a placeholder for future
	/// scope-specific lookups (e.g., resource URI or language ID for
	/// overrides) but is unused in the current simplified merging logic.
	pub fn get_value(&self, section:Option<&str>, _scope_uri_components:Option<&Value>) -> Value {
		trace!(
			"[AppState ConfigAccess] get_value called: section={:?}, scope_uri_components={:?}",
			section, _scope_uri_components
		);

		if let Some(s_path) = section {
			let mut current_val = &self.data;
			for part_key in s_path.split('.') {
				if let Some(next_val) = current_val.get(part_key) {
					current_val = next_val;
				} else {
					trace!(
						"[AppState ConfigAccess] Section part '{}' not found in config for section path '{}'. \
						 Returning null.",
						part_key, s_path
					);
					// Key part not found
					return Value::Null;
				}
			}

			current_val.clone()
		} else {
			// If no section is specified, return the entire configuration data.
			self.data.clone()
		}
	}

	/// Updates the entire configuration state from a new
	/// `MergedConfigurationState` object.
	///
	/// This is typically used after configuration files are changed and
	/// re-merged.
	pub fn update_from_new_state(&mut self, new_state:MergedConfigurationState) {
		info!("[AppState ConfigAccess] Updating entire merged configuration state from new state object.");
		trace!(
			"[AppState ConfigAccess] Old data items count: {}, New data items count: {}",
			self.data.as_object().map_or(0, |obj_map| obj_map.len()),
			new_state.data.as_object().map_or(0, |obj_map| obj_map.len())
		);
		self.data = new_state.data;
	}
}

// DTOs for deserializing text model changes from Cocoon.
// These match `vscode.editor.common.model.textModelEvents.ts` structures.

/// Represents a content change operation on a text model.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RpcModelContentChangeDto {
	// The range of text to be replaced.
	range:RpcRangeDto,

	// `rangeOffset` and `rangeLength` are not directly used in the simple
	// line-based Vec<String> model but are part of VS Code's DTO.
	// range_offset: u32,

	// range_length: u32,

	// The new text to insert.
	text:String,
}

/// Represents a range in a text model (0-indexed from Cocoon).
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RpcRangeDto {
	// 0-indexed line number
	start_line_number:usize,

	// 0-indexed column
	start_column:usize,

	// 0-indexed line number
	end_line_number:usize,

	// 0-indexed column
	end_column:usize,
}

/// Represents the state of an open text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentState {
	#[serde(with = "url_serde_helper")]
	// URI of the document
	pub uri: Url,

	// Language identifier (e.g., "typescript")
	pub language_id:String,

	// Version number, incremented on each change
	pub version:i64,

	// Content of the document as a vector of lines
	pub lines:Vec<String>,

	// End-of-line sequence (e.g., "\n" or "\r\n")
	pub eol:String,

	// True if the document has unsaved changes
	pub is_dirty:bool,

	// Detected or specified encoding (e.g., "utf8")
	pub encoding:String,
}

impl DocumentState {
	/// Gets the full text content of the document by joining lines with EOL.
	pub fn get_text_content(&self) -> String { self.lines.join(&self.eol) }

	/// Applies a set of changes (typically from Cocoon) to the document's
	/// content.
	///
	/// This method updates `lines`, `version`, and `is_dirty` state.
	/// It expects changes to be ordered such that they can be applied
	/// sequentially.
	///
	/// # Arguments
	/// * `new_version_id` - The version ID associated with these changes.
	/// * `changes_dto_val` - A `serde_json::Value` representing an array of
	///   `RpcModelContentChangeDto`s, or potentially a full text string for
	///   replacement.
	///
	/// # Returns
	/// * `Ok(())` on successful application of changes.
	/// * `Err(String)` if changes are stale, malformed, or apply out of bounds.
	pub fn apply_document_content_changes(&mut self, new_version_id:i64, changes_dto_val:&Value) -> Result<(), String> {
		// Prevent applying stale changes if the new version is not greater.
		if new_version_id <= self.version && changes_dto_val.as_array().map_or(false, |arr| !arr.is_empty()) {
			warn!(
				"[DocState ApplyChanges] Ignoring stale content changes for URI '{}': Incoming version {} <= Current \
				 version {}. Changes: {:?}",
				self.uri, new_version_id, self.version, changes_dto_val
			);
			// Stale changes are not an error, just ignored.
			return Ok(());
		}

		if new_version_id <= self.version && changes_dto_val.as_array().map_or(true, |arr| arr.is_empty()) {
			debug!(
				"[DocState ApplyChanges] Ignoring stale or no-op version bump for URI '{}': Incoming version {} <= \
				 Current version {}. No content changes.",
				self.uri, new_version_id, self.version
			);
			return Ok(());
		}

		debug!(
			"[DocState ApplyChanges] Applying V{} (Current V{}) for {}. Incoming changes DTO: {:?}",
			new_version_id, self.version, self.uri, changes_dto_val
		);

		// Attempt to deserialize into a Vec of RpcModelContentChangeDto
		let rpc_changes_vec:Vec<RpcModelContentChangeDto> = match serde_json::from_value(changes_dto_val.clone()) {
			Ok(changes) => changes,

			Err(deser_error) => {
				// Fallback: if `changes_dto_val` is just a string, treat it as a full text
				// replacement.
				if let Some(full_text_str) = changes_dto_val.as_str() {
					info!(
						"[DocState ApplyChanges] Received full text string for V{}. Replacing content of document {}.",
						new_version_id, self.uri
					);
					let (new_lines_vec, new_eol_str) = analyze_text_lines_and_eol_for_document_state(full_text_str);
					self.lines = new_lines_vec;
					// Assume sidecar provides correct EOL if sending full text
					self.eol = new_eol_str;
					self.version = new_version_id;
					// Full replacement makes it dirty
					self.is_dirty = true;
					return Ok(());
				}

				// If not a string and not a valid array of changes, but version is newer,

				// it might be a version bump without content changes (e.g., empty changes
				// array).
				if changes_dto_val.as_array().map_or(true, |arr| arr.is_empty()) && new_version_id > self.version {
					debug!(
						"[DocState ApplyChanges] Applying version bump (V{} -> V{}) with no content changes for {}.",
						self.version, new_version_id, self.uri
					);
					self.version = new_version_id;
					// `is_dirty` might be set by a separate notification ($acceptDirtyStateChanged)
					return Ok(());
				}

				// If all fallbacks fail, return deserialization error.
				return Err(format!(
					"Invalid RpcModelContentChangeDto structure for document {}: {}",
					self.uri, deser_error
				));
			},
		};

		if rpc_changes_vec.is_empty() && new_version_id > self.version {
			debug!(
				"[DocState ApplyChanges] Version bump (V{} -> V{}) with empty changes array for {}. No content \
				 modification.",
				self.version, new_version_id, self.uri
			);
			self.version = new_version_id;
			return Ok(());
		}

		// Apply changes sequentially as they are typically sent by VS Code's model in
		// an order that allows for this.
		// TODO: This sequential application is a simplified model. A more robust
		// implementation       might use a proper text buffer/rope structure or apply
		// changes in reverse order       of range to avoid index shifting issues if
		// changes are not guaranteed to be       non-overlapping and ordered
		// correctly by the sender. For now, assume VS Code's       change DTOs are
		// safe for sequential application.
		for change_op in rpc_changes_vec {
			// Convert 0-indexed DTO line/column to 0-indexed Vec/String indices for Rust.
			let start_line_idx = change_op.range.start_line_number;
			// Column is char-based index
			let mut start_col_char_idx = change_op.range.start_column;
			let end_line_idx = change_op.range.end_line_number;
			let mut end_col_char_idx = change_op.range.end_column;

			trace!(
				"[DocState ApplyChanges] Applying single change: range L{}(C{})-L{}(C{}), text: '{}...'",
				// Log as 1-based for human readability
				start_line_idx + 1,
				start_col_char_idx + 1,
				end_line_idx + 1,
				end_col_char_idx + 1,
				change_op.text.chars().take(20).collect::<String>()
			);

			// --- Boundary Checks and Index Clamping ---
			// Ensure line indices are within bounds of `self.lines`.
			// `start_line_idx` can be `self.lines.len()` for appending to a new line at the
			// end.
			if start_line_idx > self.lines.len() || end_line_idx > self.lines.len() {
				error!(
					"[DocState ApplyChanges] Invalid line range for document {}: Range L{}-L{} exceeds line count {}. \
					 Change: {:?}. Skipping this change.",
					self.uri,
					start_line_idx + 1,
					end_line_idx + 1,
					self.lines.len(),
					change_op
				);
				// Skip this invalid change
				continue;
			}

			// Clamp column indices to be within the character count of their respective
			// lines.
			if start_line_idx < self.lines.len() {
				start_col_char_idx = std::cmp::min(start_col_char_idx, self.lines[start_line_idx].chars().count());
			} else if start_line_idx == self.lines.len() && start_col_char_idx != 0 {
				// Appending to a new line, start_col must be 0.
				error!(
					"[DocState ApplyChanges] Invalid start column {} for append to new line (line {}) for document \
					 {}. Change: {:?}. Skipping.",
					start_col_char_idx + 1,
					start_line_idx + 1,
					self.uri,
					change_op
				);
				continue;
			}

			if end_line_idx < self.lines.len() {
				end_col_char_idx = std::cmp::min(end_col_char_idx, self.lines[end_line_idx].chars().count());
			} else if end_line_idx == self.lines.len() && end_col_char_idx != 0 {
				// Range ends on a new line after the current last line; end_col must be 0.
				error!(
					"[DocState ApplyChanges] Invalid end column {} for range ending on new line (line {}) for \
					 document {}. Change: {:?}. Skipping.",
					end_col_char_idx + 1,
					end_line_idx + 1,
					self.uri,
					change_op
				);
				continue;
			}

			// --- Apply Change ---
			let text_to_insert_lines:Vec<String> = change_op.text.split(&self.eol).map(String::from).collect();

			if start_line_idx == end_line_idx {
				// Single-line change (insertion or replacement within one line)
				if start_line_idx >= self.lines.len() {
					// Adding new line(s) at the very end of the document.
					if start_line_idx == self.lines.len() && start_col_char_idx == 0 && end_col_char_idx == 0 {
						self.lines.extend(text_to_insert_lines);
					} else {
						error!(
							"[DocState ApplyChanges] Attempting single-line change on non-existent line {} or with \
							 invalid columns for document {}. Change: {:?}. Skipping.",
							start_line_idx + 1,
							self.uri,
							change_op
						);
						continue;
					}
				} else {
					// Modify an existing line.
					let line_to_modify = &mut self.lines[start_line_idx];
					let original_line_tail:String = line_to_modify.chars().skip(end_col_char_idx).collect();
					let mut new_line_content:String = line_to_modify.chars().take(start_col_char_idx).collect();

					if text_to_insert_lines.len() == 1 {
						// Inserted text is a single line (no new EOLs within it).
						new_line_content.push_str(&text_to_insert_lines[0]);
						new_line_content.push_str(&original_line_tail);
						*line_to_modify = new_line_content;
					} else {
						// Inserted text is multi-line, effectively splitting the current line.
						new_line_content.push_str(&text_to_insert_lines[0]);
						// Update the first part of the split line.
						*line_to_modify = new_line_content;

						// Insert the intermediate new lines from `text_to_insert_lines`.
						// These are inserted *after* `start_line_idx`.
						for i in 1..text_to_insert_lines.len() - 1 {
							self.lines.insert(start_line_idx + i, text_to_insert_lines[i].clone());
						}

						// Add the last line of the inserted text, prepended to the original line's
						// tail.
						let last_inserted_line_part = text_to_insert_lines.last().unwrap().clone();
						self.lines.insert(
							start_line_idx + text_to_insert_lines.len() - 1,
							last_inserted_line_part + &original_line_tail,
						);
					}
				}
			} else {
				// Multi-line change (replacing a range spanning multiple lines).
				if start_line_idx >= self.lines.len() {
					error!(
						"[DocState ApplyChanges] Attempting multi-line change starting on non-existent line {} for \
						 document {}. Change: {:?}. Skipping.",
						start_line_idx + 1,
						self.uri,
						change_op
					);
					continue;
				}

				// Preserve the part of the start line *before* the change range.
				let first_line_prefix:String = self.lines[start_line_idx].chars().take(start_col_char_idx).collect();
				// Preserve the part of the end line *after* the change range.
				let last_line_suffix:String = if end_line_idx < self.lines.len() {
					self.lines[end_line_idx].chars().skip(end_col_char_idx).collect()
				} else {
					// Range extends to or beyond the end of the document.
					String::new()
				};

				// Construct the new content for the start line by combining prefix and first
				// line of inserted text.
				let mut modified_start_line_content = first_line_prefix;
				modified_start_line_content.push_str(&text_to_insert_lines[0]);

				// If the inserted text is just one line, combine it with `last_line_suffix` on
				// `start_line_idx`.
				if text_to_insert_lines.len() == 1 {
					modified_start_line_content.push_str(&last_line_suffix);
					self.lines[start_line_idx] = modified_start_line_content;
				} else {
					// Inserted text is multi-line.
					// Set the modified start line.
					self.lines[start_line_idx] = modified_start_line_content;

					// Insert intermediate lines from `text_to_insert_lines`.
					for i in 1..text_to_insert_lines.len() - 1 {
						self.lines.insert(start_line_idx + i, text_to_insert_lines[i].clone());
					}

					// Insert the last line of `text_to_insert_lines`, combined with
					// `last_line_suffix`.
					let final_inserted_line_content = text_to_insert_lines.last().unwrap().clone() + &last_line_suffix;
					self.lines
						.insert(start_line_idx + text_to_insert_lines.len() - 1, final_inserted_line_content);
				}

				// Remove the original lines that were spanned by the multi-line range.
				// This needs to account for lines possibly added/removed by the insertion
				// above. The lines to remove are from the original `start_line_idx + 1` up
				// to `end_line_idx`.
				let num_original_lines_in_deleted_range_after_start_line = end_line_idx - start_line_idx;

				if num_original_lines_in_deleted_range_after_start_line > 0 {
					// Calculate the actual starting index for removal, which is after all newly
					// inserted lines (or after the modified start line if the insertion was a
					// single line).
					let removal_start_actual_idx = start_line_idx + std::cmp::max(1, text_to_insert_lines.len());
					// Ensure removal_start_actual_idx is within current bounds.
					if removal_start_actual_idx < self.lines.len() {
						let removal_end_actual_idx = std::cmp::min(
							// Don't go past the end of current lines
							self.lines.len(),
							removal_start_actual_idx + num_original_lines_in_deleted_range_after_start_line,
						);
						if removal_start_actual_idx < removal_end_actual_idx {
							self.lines.drain(removal_start_actual_idx..removal_end_actual_idx);
						}
					} else if removal_start_actual_idx == self.lines.len()
						&& num_original_lines_in_deleted_range_after_start_line > 0
					{
						// This means we inserted lines, and the original range to delete might now be
						// entirely beyond the current document end or just at the end. This is fine.
						trace!(
							"[DocState ApplyChanges] Multi-line delete range is at/beyond end after insertions for \
							 {}. No lines drained.",
							self.uri
						);
					} else if num_original_lines_in_deleted_range_after_start_line > 0 {
						// This case (removal_start_actual_idx > self.lines.len()) should be rare if
						// logic is correct.
						debug!(
							"[DocState ApplyChanges] Calculated removal start index {} is out of bounds (lines: {}). \
							 No lines drained for multi-line delete. URI: {}",
							removal_start_actual_idx,
							self.lines.len(),
							self.uri
						);
					}
				}
			}
		}

		self.version = new_version_id;
		// Any content change implies dirty status.
		self.is_dirty = true;
		Ok(())
	}
}

/// Represents the state of an active integrated terminal.
/// (Fields related to PTY handles and tasks are `serde(skip)` as they are not
/// persistent state.)
// `Default` removed as some fields (tasks) are not easily defaultable.
#[derive(Debug, Clone)]
pub struct TerminalState {
	// Unique ID for the terminal instance
	pub id:u64,

	// Display name of the terminal
	pub name:String,

	// Path to the shell executable
	pub shell_path:String,

	// Arguments for the shell
	pub shell_args:Vec<String>,

	// Current working directory for the shell
	pub cwd:Option<PathBuf>,

	// Environment variables for the shell
	pub env:Option<HashMap<String, String>>,

	// OS-level Process ID of the shell
	pub os_process_id:Option<u32>,

	// True if this is a PTY-backed terminal
	pub is_pty:bool,

	/// Sender part of an MPSC channel to send input strings to the PTY writer
	/// task. This sender can be cloned (e.g., by
	/// `handle_sendText_to_terminal`).
	#[serde(skip)]
	pub pty_input_tx:Option<TokioMpsc::Sender<String>>,

	/// Join handle for the asynchronous task that reads from the PTY master's
	/// output. Wrapped to allow the handle to be taken and aborted once.
	#[serde(skip)]
	pub reader_task_handle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,

	/// Join handle for the asynchronous task that waits for the PTY child
	/// process to exit.
	#[serde(skip)]
	pub process_wait_handle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
	// TODO: Consider storing the `PtyChild` or `MasterPty` handles directly if needed for
	//       more advanced operations like resizing the PTY, though `portable-pty`'s design
	//       might make this tricky with cloned reader/writer parts. Resizing typically
	//       needs the `MasterPty` object.
}

impl TerminalState {
	/// Creates a new `TerminalState` instance from provided options.
	///
	/// `pty_input_tx`, `reader_task_handle`, and `process_wait_handle` are
	/// initialized to `None` and are set later during PTY setup.
	pub fn new(
		id:u64,

		name:String,

		// From ICreateTerminalOptions
		options_val:&Value,

		default_shell_path:String,
	) -> Self {
		let shell_path_opt_str = options_val.get("shellPath").and_then(Value::as_str);
		let final_shell_path = shell_path_opt_str.map_or(default_shell_path, String::from);

		let shell_args_val = options_val.get("shellArgs");
		let final_shell_args_vec = if let Some(s_arg_str) = shell_args_val.and_then(Value::as_str) {
			// If `shellArgs` is a single string, split it (basic space splitting for now).
			// TODO: Implement more robust shell argument parsing if complex string args are
			// common.       VS Code's `shellArgs` is typically `string[]` or `string`
			// (for Windows cmd).
			s_arg_str.split_whitespace().map(String::from).collect()
		} else if let Some(arr_val) = shell_args_val.and_then(Value::as_array) {
			arr_val
				.iter()
				 // Take only string elements
				.filter_map(Value::as_str)
				.map(String::from)
				.collect()
		} else {
			// Default to no arguments
			Vec::new()
		};

		let cwd_opt_path = options_val.get("cwd").and_then(Value::as_str).map(PathBuf::from);

		let env_vars_opt_map = if let Some(env_map_val) = options_val.get("env").and_then(Value::as_object) {
			let mut env_map = HashMap::new();
			for (k, v_val) in env_map_val {
				if let Some(v_str) = v_val.as_str() {
					env_map.insert(k.clone(), v_str.to_string());
				} else if v_val.is_null() {
					// VS Code allows `null` to unset an env var inherited from parent.
					// `CommandBuilder::env_remove` could handle this if `portable-pty` supports it,

					// or if not inheriting parent env. For now, `null` values are ignored.
					warn!(
						"[TerminalState new] Ignoring null value for env var '{}'; unsetting inherited env vars via \
						 null is not directly supported by current PTY setup. Variable will be inherited if present \
						 in parent.",
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

			// Set after PTY process spawns
			os_process_id:None,

			// Default to PTY-backed terminal
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
	// Display name of the channel
	pub name:String,

	// Optional language ID for syntax highlighting
	pub language_id:Option<String>,

	// Current content buffer of the channel
	pub buffer:String,

	// True if the channel is currently visible in the UI
	pub visible:bool,
}

impl OutputChannelState {
	/// Creates a new `OutputChannelState`.
	pub fn new(name:&str, language_id:Option<String>) -> Self {
		Self {
			name:name.to_string(),

			language_id,

			// Initialize with empty buffer
			buffer:String::new(),

			// Initially not visible
			visible:false,
		}
	}
}

/// Enum representing the types of language feature providers extensions can
/// register. Mirrors `vscode.languages` provider types.
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

	// General DocumentFormattingEditProvider
	Formatting,

	// DocumentRangeFormattingEditProvider
	RangeFormatting,

	OnTypeFormatting,

	Rename,

	DocumentLink,

	// DocumentColorProvider
	Color,

	FoldingRange,

	SelectionRange,

	CallHierarchy,

	TypeHierarchy,

	LinkedEditingRange,

	InlayHints,
	// TODO (Feature): Add `SignatureHelp` if its metadata (SignatureHelpProviderMetadataDto)
	//       is distinct enough or needs special handling during registration/query.
}

/// Represents a registered language feature provider.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderRegistration {
	// Unique handle generated by Mountain for this registration
	pub handle:u32,

	// Type of provider (e.g., Hover, Completion)
	pub provider_type:LanguageProviderType,

	// `DocumentSelector` (JSON Value) defining when this provider applies
	pub selector:Value,

	// ID of the sidecar (e.g., "cocoon-main") that registered this provider
	pub sidecar_id:String,

	// Optional metadata specific to certain provider types, mirroring VS Code's
	// ProviderMetadata DTOs.
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
	pub signature_help_metadata:Option<Value>, /* For SignatureHelpProvider (e.g., SignatureHelpProviderMetadataDto)
	                                            * TODO: Add other provider-specific metadata fields as needed (e.g.,
	                                            *
	                                            *
	                                            * for CodeLens, Links). */
}

/// Represents the metadata of a scanned extension, derived from its
/// `package.json`. Mirrors `vscode.IExtensionDescription`.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDescriptionState {
	/// Extension identifier DTO: `{ value: "publisher.name", uuid?: string }`
	pub identifier:Value,

	// From package.json "name"
	pub name:String,

	// From package.json "version"
	pub version:String,

	// From package.json "publisher"
	pub publisher:String,

	// Engine compatibility, e.g., `{ "vscode": "^1.80.0" }`
	pub engines:Value,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Entry point for Node.js extension host (relative path)
	pub main: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Entry point for Web extension host (relative path)
	pub browser: Option<String>,

	#[serde(rename = "type", skip_serializing_if = "Option::is_none")]
	// "commonjs" or "module" (for ESM support)
	pub module_type: Option<String>,

	// Defaults to false if not present
	#[serde(default)]
	// True if this is a built-in Mountain/Land extension
	pub is_builtin: bool,

	#[serde(default)]
	// True if running in extension development mode
	pub is_under_development: bool,

	/// URI (as `UriComponents` DTO) to the extension's installation location.
	pub extension_location:Value,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Events that trigger extension activation
	pub activation_events: Option<Vec<String>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub contributes:Option<Value>, /* The entire 'contributes' object from package.json
	                                * TODO: Add other relevant fields from IExtensionDescription as needed:
	                                *       `displayName`, `description`, `categories`, `keywords`,
	                                *
	                                *
	                                * `extensionDependencies`, etc. */
}

// --- Central Application State Struct ---
/// Holds all shared, mutable application state for Mountain.
///
/// This struct is managed by Tauri and accessed via
/// `app_handle.state::<AppState>()`. All fields that can be mutated
/// concurrently are wrapped in `Arc<StdMutex<_>>` or appropriate atomic types
/// for thread safety.
// Clone is needed for AppState to be managed by Tauri's `State`.
#[derive(Clone)]
pub struct AppState {
	// Workspace State
	pub workspace_folders:Arc<StdMutex<Vec<WorkspaceFolderState>>>,

	// Path to .code-workspace file
	pub workspace_config_path:Arc<StdMutex<Option<PathBuf>>>,

	// Workspace trust state
	pub is_trusted:Arc<AtomicBool>,

	// Configuration State
	// Merged effective configuration
	pub configuration:Arc<StdMutex<MergedConfigurationState>>,

	// Extension Storage (Mementos)
	pub global_memento:Arc<StdMutex<MementoStorageMap>>,

	// Resolved path to globalStorage.json
	pub global_memento_path:PathBuf,

	pub workspace_memento:Arc<StdMutex<MementoStorageMap>>,

	// Path to workspace storage.json
	pub workspace_memento_path:Arc<StdMutex<Option<PathBuf>>>,

	// Command System
	pub command_registry:Arc<StdMutex<CommandRegistryMap>>,

	// Diagnostics
	pub diagnostics_map:Arc<StdMutex<DiagnosticsStorageMap>>,

	// Open Documents
	pub open_documents:Arc<StdMutex<OpenDocumentMap>>,

	// Output Channels
	pub output_channels:Arc<StdMutex<OutputChannelStorageMap>>,

	// Language Features
	pub language_providers:Arc<StdMutex<LanguageProviderRegistrationMap>>,

	// Counter for unique provider handles
	pub next_provider_handle:Arc<AtomicU32>,

	// Extension Management
	/// All extensions Mountain knows about (e.g., scanned from disk). Key is
	/// `publisher.name`.
	pub scanned_extensions:Arc<StdMutex<ScannedExtensionMetadataMap>>,

	/// Configuration for proposed APIs. Key: extensionId or `*`, Value: list of
	/// proposal names.
	pub enabled_proposed_apis:Arc<StdMutex<EnabledProposedApisConfigMap>>,

	/// Paths to directories where pre-bundled/scanned extensions are located.
	/// Populated during startup (e.g., in `main.rs` after `AppHandle` is
	/// available).
	pub extension_scan_paths:Arc<StdMutex<Vec<PathBuf>>>,

	// Integrated Terminals
	pub active_terminals:Arc<StdMutex<ActiveTerminalMap>>,

	// Counter for unique terminal IDs
	pub next_terminal_id:Arc<AtomicU64>,

	// UI Interaction State
	/// Stores `oneshot::Sender` channels for pending UI requests made from
	/// async Rust (e.g., `UiProvider` effects in `environment.rs`) to the
	/// Tauri frontend (Sky), awaiting a response via `sky_resolves_ui_request`
	/// command.
	pub pending_ui_requests:Arc<StdMutex<PendingUiRequestChannelMap>>,
}

// --- Helper Functions (module-private, used in AppState::default) ---

/// Helper to determine the file path for persistent extension storage
/// (memento).
///
/// Paths are modeled after VS Code's storage layout:
/// - Global: `[app_data_dir]/User/globalStorage.json`
/// - Workspace:
///   `[app_data_dir]/User/workspaceStorage/[workspace_id_hash]/storage.json`
///
/// # Arguments
/// * `app_data_dir` - The base application data directory.
/// * `is_global_scope` - True for global memento, false for workspace memento.
/// * `workspace_id_str` - A unique identifier for the workspace (used if
///   `is_global_scope` is false). Can be empty if no workspace is active for
///   initial path resolution.
///
/// # Returns
/// A `PathBuf` to the memento JSON file.
fn resolve_memento_storage_file_path(app_data_dir:&Path, is_global_scope:bool, workspace_id_str:&str) -> PathBuf {
	// VS Code typically uses a "User" subdirectory within the app data directory
	// for user-specific state.
	let user_storage_base_path = app_data_dir.join("User");

	if is_global_scope {
		user_storage_base_path.join("globalStorage.json")
	} else {
		// Sanitize workspace_id to make it a valid directory name component.
		// Replace non-alphanumeric characters (except hyphens/underscores) with
		// underscores. TODO: Consider using a cryptographic hash (e.g., SHA256) of
		// the canonical workspace       URI or .code-workspace path for a more robust
		// and collision-resistant ID.
		let sanitized_workspace_id_segment =
			workspace_id_str.replace(|c:char| !c.is_alphanumeric() && c != '-' && c != '_', "_");

		// Path: AppData/User/workspaceStorage/<sanitized_workspace_id>/storage.json
		user_storage_base_path
			.join("workspaceStorage")
			.join(sanitized_workspace_id_segment)
			.join("storage.json")
	}
}

/// Helper function to load initial memento storage data from a JSON file.
///
/// Uses blocking synchronous I/O, suitable only for initialization contexts
/// like `AppState::default()`.
///
/// # Arguments
/// * `storage_file_path` - The `Path` to the memento JSON file.
///
/// # Returns
/// A `MementoStorageMap` (HashMap). Returns an empty map if the file is not
/// found, empty, or fails to parse.
fn load_initial_memento_storage_from_disk(storage_file_path:&Path) -> MementoStorageMap {
	if !storage_file_path.exists() {
		debug!(
			"[AppState Init MementoLoad] Storage file not found at '{}', creating empty map.",
			storage_file_path.display()
		);
		return HashMap::new();
	}

	debug!(
		"[AppState Init MementoLoad] Attempting to load memento storage from: {}",
		storage_file_path.display()
	);

	match fs::read_to_string(storage_file_path) {
		Ok(json_content_str) => {
			if json_content_str.trim().is_empty() {
				debug!(
					"[AppState Init MementoLoad] Storage file '{}' is empty. Returning empty map.",
					storage_file_path.display()
				);
				return HashMap::new();
			}

			match serde_json::from_str(&json_content_str) {
				Ok(parsed_map) => {
					info!(
						"[AppState Init MementoLoad] Successfully loaded {} items from storage file: {}",
						parsed_map.len(),
						storage_file_path.display()
					);
					parsed_map
				},

				Err(e_parse) => {
					error!(
						"[AppState Init MementoLoad] Failed to parse JSON from storage file '{}'. Returning empty \
						 map. Error: {}",
						storage_file_path.display(),
						e_parse
					);
					// TODO: Consider backup/recovery mechanism if a storage file gets corrupted.
					HashMap::new()
				},
			}
		},

		Err(e_read) => {
			// Log error only if it's not a "NotFound" error (which is handled by the
			// `exists()` check above).
			if e_read.kind() != std::io::ErrorKind::NotFound {
				error!(
					"[AppState Init MementoLoad] Failed to read storage file '{}'. Returning empty map. Error: {}",
					storage_file_path.display(),
					e_read
				);
			} else {
				// This case should be caught by `!exists()` but good for robustness.
				debug!(
					"[AppState Init MementoLoad] Storage file confirmed not found during read attempt: {}",
					storage_file_path.display()
				);
			}

			HashMap::new()
		},
	}
}

/// Helper function to split text into lines and detect EOL.
/// Renamed to be specific to its use in `DocumentState` context.
/// This is a simplified version;
/// `handlers::documents::analyze_text_lines_and_eol` might be more robust.
pub fn analyze_text_lines_and_eol_for_document_state(text:&str) -> (Vec<String>, String) {
	// Simplified EOL detection for DocumentState initialization or full
	// replacement. Prefers \r\n > \n. Pure \r is treated as \n.
	let detected_eol = if text.contains("\r\n") {
		"\r\n"
	} else {
		// Default to LF, also handles pure LF or pure CR (normalized to LF)
		"\n"
	};
	let lines = text.split(detected_eol).map(String::from).collect();
	(lines, detected_eol.to_string())
}

// --- Default Implementation for AppState ---
impl Default for AppState {
	/// Initializes the `AppState` with default values and loads persistent
	/// state.
	///
	/// This function runs synchronously during Tauri application setup.
	/// It performs crucial initializations like determining application data
	/// paths, loading global memento storage, and registering native Mountain
	/// commands.
	fn default() -> Self {
		info!("[AppState Default] Initializing default application state...");

		// Determine Application Data Directory.
		// Uses package name from Cargo.toml (e.g., "land_mountain") to create a unique
		// subdirectory. TODO: Consider making the application name for directory
		// structure configurable       (e.g., "LandEditor") rather than relying
		// directly on `CARGO_PKG_NAME` if       the crate name might change or is not
		// user-friendly for paths.
		let app_name_for_paths = env!("CARGO_PKG_NAME");
		// OS-specific config directory (e.g., ~/.config)
		let app_data_dir_path = dirs::config_dir().map(|p| p.join(app_name_for_paths)).unwrap_or_else(|| {
			// Fallback if system config directory cannot be determined.
			warn!(
				"[AppState Default] Could not determine system config/data directory. Using relative path \
				 '.{}-appdata' in current working directory.",
				app_name_for_paths
			);
			PathBuf::from(format!(".{}-appdata", app_name_for_paths))
		});

		// Ensure the base application data directory exists.
		if !app_data_dir_path.exists() {
			if let Err(e_create_dir) = fs::create_dir_all(&app_data_dir_path) {
				// This is a critical failure if the app cannot create its data directory.
				error!(
					"[AppState Default] CRITICAL: Failed to create application data directory at '{}': {}. \
					 Persistence may fail.",
					app_data_dir_path.display(),
					e_create_dir
				);
				// Application might still run but with storage/config issues.
			}
		}

		// Load Global Memento Storage
		// true for global, empty wsId
		let global_memento_file_path = resolve_memento_storage_file_path(&app_data_dir_path, true, "");
		debug!(
			"[AppState Default] Global memento storage path resolved to: {}",
			global_memento_file_path.display()
		);
		let initial_global_memento_map = load_initial_memento_storage_from_disk(&global_memento_file_path);

		// Workspace memento is initialized empty. Its path is `None` until a workspace
		// is opened.
		let initial_workspace_memento_map = HashMap::new();
		let workspace_memento_file_path_arc_mutex = Arc::new(StdMutex::new(None));

		// Initialize Command Registry & Register Native Mountain Commands
		let mut initial_command_registry_map = HashMap::new();
		info!("[AppState Default] Registering native Mountain commands...");
		// Use the registration helper from `handlers::commands`.
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
		// TODO (Native Commands): Add more native commands here as the application
		// grows       (e.g., for file operations, settings UI interactions, window
		// management).

		// Initialize other state fields.
		let scanned_extensions_map_arc_mutex = Arc::new(StdMutex::new(HashMap::new()));
		let enabled_proposed_apis_map_arc_mutex = Arc::new(StdMutex::new(HashMap::new()));
		// `extension_scan_paths` is initialized empty. It's populated later in
		// `main.rs` setup after `AppHandle` is available to resolve bundled resource
		// paths.
		let extension_scan_paths_arc_mutex = Arc::new(StdMutex::new(Vec::new()));

		info!(
			"[AppState Default] Default state initialization complete. Global Memento Path: '{}'. App Data Dir: '{}'",
			global_memento_file_path.display(),
			app_data_dir_path.display()
		);

		AppState {
			workspace_folders:Arc::new(StdMutex::new(Vec::new())),

			configuration:Arc::new(StdMutex::new(MergedConfigurationState::default())),

			// Default to not trusted
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

			// Start handles at 1
			next_provider_handle:Arc::new(AtomicU32::new(1)),

			scanned_extensions:scanned_extensions_map_arc_mutex,

			enabled_proposed_apis:enabled_proposed_apis_map_arc_mutex,

			extension_scan_paths:extension_scan_paths_arc_mutex,

			active_terminals:Arc::new(StdMutex::new(HashMap::new())),

			// Start terminal IDs at 1
			next_terminal_id:Arc::new(AtomicU64::new(1)),

			pending_ui_requests:Arc::new(StdMutex::new(HashMap::new())),
		}
	}
}

// --- AppState Methods ---
impl AppState {
	/// Helper method to determine a unique identifier string for the current
	/// workspace.
	///
	/// This ID is used for scoping workspace-specific storage (mementos) and
	/// potentially other workspace-specific state.
	/// - If a `.code-workspace` file is loaded, its filename is used.
	/// - Otherwise, if workspace folders are open, the path of the first folder
	///   is used (sanitized).
	/// - If no workspace or folders, returns "NO_WORKSPACE".
	///
	/// # Returns
	/// * `Ok(String)` with the workspace ID.
	/// * `Err(String)` if a Mutex lock fails (poisoned).
	///
	/// # Panics
	/// Can panic if `unwrap()` is called on a poisoned lock, though `map_err`
	/// is used here.
	pub fn get_workspace_id_string(&self) -> Result<String, String> {
		// Prefer .code-workspace file path for ID if available.
		let config_path_guard = self
			.workspace_config_path
			.lock()
			.map_err(|e| format!("Lock error on workspace_config_path: {}", e))?;
		if let Some(config_path) = config_path_guard.as_ref() {
			// Using the filename of the .code-workspace file.
			// TODO: For more robust uniqueness, consider a hash of the canonicalized full
			// path.       e.g.,

			// sha256(config_path.canonicalize().unwrap_or(config_path.clone()).
			// to_string_lossy())
			return Ok(config_path
				.file_name()
				 // Should always have a filename if path is valid
				.unwrap_or_default()
				.to_string_lossy()
				.into_owned());
		}

		// Release lock before acquiring another
		drop(config_path_guard);

		// If no .code-workspace, use the path of the first workspace folder.
		let folders_guard = self
			.workspace_folders
			.lock()
			.map_err(|e| format!("Lock error on workspace_folders: {}", e))?;
		if let Some(first_folder) = folders_guard.first() {
			// Using sanitized URI path. A hash of the canonical URI path would be more
			// robust. Replace non-alphanumeric characters to make it a safer directory
			// name component.
			return Ok(first_folder
				.uri
				.path()
				.replace(|c:char| !c.is_alphanumeric() && c != '/' && c != '\\', "_"));
		}

		// If no workspace config and no folders, effectively no workspace.
		Ok("NO_WORKSPACE".to_string())
	}

	/// Updates the workspace memento file path based on the current workspace
	/// ID and reloads its content from disk.
	///
	/// This should be called after `workspace_folders` and/or
	/// `workspace_config_path` are set or changed (e.g., when a workspace is
	/// opened).
	///
	/// # Arguments
	/// * `app_data_dir` - The resolved application data directory path.
	///
	/// # Returns
	/// * `Ok(())` on success.
	/// * `Err(String)` if determining workspace ID or locking state fails.
	pub fn update_workspace_memento_path_and_reload(&self, app_data_dir:&Path) -> Result<(), String> {
		let workspace_id_str = self.get_workspace_id_string()?;
		if workspace_id_str == "NO_WORKSPACE" {
			// If no workspace, clear the path and memento.
			let mut path_guard = self
				.workspace_memento_path
				.lock()
				.map_err(|e| format!("Lock error (workspace memento path for clear): {}", e))?;
			if path_guard.is_some() {
				info!("[AppState Memento] No active workspace, clearing workspace memento path and data.");
				*path_guard = None;
				let mut memento_data_guard = self
					.workspace_memento
					.lock()
					.map_err(|e| format!("Lock error (workspace memento data for clear): {}", e))?;
				memento_data_guard.clear();
			}

			return Ok(());
		}

		// false for workspace scope
		let new_memento_file_path = resolve_memento_storage_file_path(app_data_dir, false, &workspace_id_str);

		let mut path_guard = self
			.workspace_memento_path
			.lock()
			.map_err(|e| format!("Lock error (workspace memento path for update): {}", e))?;

		// Check if the path actually needs to change.
		if path_guard.as_ref() != Some(&new_memento_file_path) {
			info!(
				"[AppState Memento] Updating workspace memento path to: {}",
				new_memento_file_path.display()
			);
			// Ensure parent directory for the new memento file exists.
			if let Some(parent_dir) = new_memento_file_path.parent() {
				if !parent_dir.exists() {
					if let Err(e_create) = fs::create_dir_all(parent_dir) {
						error!(
							"[AppState Memento] Failed to create directory for workspace memento at '{}': {}. Load \
							 may fail.",
							parent_dir.display(),
							e_create
						);
						// Proceed to set path;
						// load_initial_memento_storage_from_disk will handle
						// non-existent file.
					}
				}
			}

			// Update the path
			*path_guard = Some(new_memento_file_path.clone());

			// When path changes, reload the workspace memento content from the new file.
			debug!(
				"[AppState Memento] Reloading workspace memento content from new path: {}",
				new_memento_file_path.display()
			);
			let new_memento_content_map = load_initial_memento_storage_from_disk(&new_memento_file_path);
			let mut memento_data_guard = self
				.workspace_memento
				.lock()
				.map_err(|e| format!("Lock error (workspace memento data for reload): {}", e))?;
			*memento_data_guard = new_memento_content_map;
		}

		Ok(())
	}

	/// Helper method to determine the display name for the current workspace.
	///
	/// - If a `.code-workspace` file is loaded, its name (without extension) is
	///   used.
	/// - Otherwise, the name of the first workspace folder is used.
	/// - If neither, defaults to "Untitled Workspace".
	///
	/// # Returns
	/// * `Ok(String)` with the workspace display name.
	/// * `Err(String)` if a Mutex lock fails.
	pub fn get_workspace_name(&self) -> Result<String, String> {
		let config_path_guard = self
			.workspace_config_path
			.lock()
			.map_err(|e| format!("Lock error (config path for workspace name): {}", e))?;
		Ok(match config_path_guard.as_ref().and_then(|p| p.file_stem()) {
			Some(stem) => stem.to_string_lossy().into_owned(),

			None => {
				// Release lock before acquiring another
				drop(config_path_guard);
				let folders_guard = self
					.workspace_folders
					.lock()
					.map_err(|e| format!("Lock error (folders for workspace name): {}", e))?;
				match folders_guard.first() {
					Some(folder) => folder.name.clone(),

					// Default if no config and no folders
					None => "Untitled Workspace".to_string(),
				}
			},
		})
	}

	/// Atomically generates the next unique handle for a language provider
	/// registration.
	pub fn get_next_provider_handle(&self) -> u32 {
		// Relaxed ordering is sufficient for a simple counter
		self.next_provider_handle.fetch_add(1, AtomicOrdering::Relaxed)
	}

	/// Scans configured `extension_scan_paths` for `package.json` files and
	/// populates `self.scanned_extensions` with metadata.
	///
	/// This method uses synchronous `std::fs` calls. For a fully async startup,
	///
	///
	/// this would need refactoring to use `tokio::fs` and potentially stream
	/// processing for directory entries. It's often called during startup where
	/// some blocking I/O might be permissible on a dedicated setup thread/task.
	pub async fn scan_extensions_and_populate_state(&self) {
		// Clone paths to avoid holding the lock on `extension_scan_paths` during
		// extensive I/O.
		let current_scan_paths_vec = {
			let guard = self.extension_scan_paths.lock().unwrap_or_else(|poisoned_err| {
				error!(
					"[AppState ExtScan] Poisoned lock on extension_scan_paths: {}. Attempting to recover.",
					poisoned_err
				);
				// Recover by taking the inner data
				poisoned_err.into_inner()
			});
			guard.clone()
		};

		info!(
			"[AppState ExtScan] Starting scan for extensions in paths: {:?}",
			current_scan_paths_vec
		);
		let mut found_extensions_map = HashMap::new();

		for scan_dir_path in current_scan_paths_vec {
			if !scan_dir_path.is_dir() {
				warn!(
					"[AppState ExtScan] Extension scan path is not a directory or does not exist: '{}'. Skipping.",
					scan_dir_path.display()
				);
				continue;
			}

			// NOTE: Synchronous directory walk (`fs::read_dir`). See TODO above regarding
			// async.
			match fs::read_dir(&scan_dir_path) {
				Ok(dir_entries) => {
					for entry_result in dir_entries {
						if let Ok(dir_entry) = entry_result {
							let extension_candidate_path = dir_entry.path();
							if extension_candidate_path.is_dir() {
								// Each subdirectory is potentially an extension folder.
								let package_json_file_path = extension_candidate_path.join("package.json");
								if package_json_file_path.is_file() {
									match fs::read_to_string(&package_json_file_path) {
										Ok(pkg_json_content_str) => {
											match serde_json::from_str::<Value>(&pkg_json_content_str) {
												Ok(pkg_json_val) => {
													// Extract required fields from package.json
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
																	.unwrap_or_else(|url_err| {
																		warn!(
																			"[AppState ExtScan] Failed to create \
																			 directory URL for extension path '{}': \
																			 {}. Using fallback.",
																			extension_candidate_path.display(),
																			url_err
																		);
																		// Fallback to a generic file URL.
																		Url::parse(&format!(
																			"file:///{}",
																			extension_candidate_path
																				.to_string_lossy()
																				.replace("\\", "/")
																		))
																		.unwrap_or_else(
																			|fallback_err| {
																				error!(
																					"[AppState ExtScan] CRITICAL: \
																					 Fallback URL parse failed for \
																					 extension path '{}': {}",
																					extension_candidate_path.display(),
																					fallback_err
																				);
																				Url::parse(
																					"file:///unknown_extension_path",


																				)
																				 // Should not happen
																				.unwrap()
																			},
																		)
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

																// MVP Assumption: Scanned extensions are "built-in" or
																// trusted by default. A more robust
																// system would differentiate based on origin or user
																// settings.
																is_builtin:true, /* TODO: Determine this
																                  * more accurately. */
																is_under_development:false, /* TODO: Parse from
																                             * package.json or dev
																                             * context. */
																extension_location:json!({

																	"scheme": ext_location_url.scheme(),


																	"authority": ext_location_url.host_str().unwrap_or(""),


																	"path": ext_location_url.path(),


																	"external": ext_location_url.to_string(),


																	"$mid": 1
																}),

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
																"[AppState ExtScan] Successfully scanned extension: {}",
																ext_id_str
															);
															found_extensions_map.insert(ext_id_str, ext_desc_state);
														} else {
															warn!(
																"[AppState ExtScan] Invalid package.json in '{}': \
																 missing one or more core string fields (name, \
																 publisher, version, engines.vscode).",
																package_json_file_path.display()
															);
														}
													} else {
														warn!(
															"[AppState ExtScan] Invalid package.json in '{}': core \
															 fields (name, publisher, version, engines) not found.",
															package_json_file_path.display()
														);
													}
												},

												Err(e_json_parse) => {
													warn!(
														"[AppState ExtScan] Failed to parse package.json content from \
														 '{}': {}. Skipping.",
														package_json_file_path.display(),
														e_json_parse
													);
												},
											}
										},

										Err(e_read_file) => {
											warn!(
												"[AppState ExtScan] Failed to read package.json file '{}': {}. \
												 Skipping.",
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
						"[AppState ExtScan] Failed to read entries in extension scan path '{}': {}. Skipping this \
						 path.",
						scan_dir_path.display(),
						e_read_dir
					)
				},
			}
		}

		// Update the main AppState.scanned_extensions map.
		if !found_extensions_map.is_empty() {
			let mut scanned_extensions_guard = self.scanned_extensions.lock().unwrap_or_else(|e| {
				error!(
					"[AppState ExtScan] Poisoned lock on scanned_extensions map for update: {}. Attempting to recover.",
					e
				);
				e.into_inner()
			});
			*scanned_extensions_guard = found_extensions_map;
			info!(
				"[AppState ExtScan] Extension scan complete. Updated `scanned_extensions` in AppState. Total count: {}",
				scanned_extensions_guard.len()
			);
		} else {
			info!("[AppState ExtScan] No extensions found in any configured scan paths.");
		}
	}

	/// Atomically generates the next unique ID for an integrated terminal
	/// instance.
	pub fn get_next_terminal_id(&self) -> u64 { self.next_terminal_id.fetch_add(1, AtomicOrdering::Relaxed) }

	// TODO (Feature): Add an `async fn initialize_after_app_handle_available(&self,

	// app_handle: &AppHandle<Wry>)`       This method would be called from
	// `main.rs`'s `.setup()` hook once the `AppHandle`       is available. It
	// would perform initializations that require `AppHandle`, such as:
	//       - Resolving `extension_scan_paths` using `app_handle.path_resolver()`.
	//       - Triggering the initial `scan_extensions_and_populate_state().await`.
	//       - Loading initial merged configuration via
	//         `handlers::config::load_and_merge_configurations_internal`.
	//       - Updating `workspace_memento_path` based on initially resolved app
	//         data directory.
	//       - Loading any last session state (e.g., open files, window layout).
}

// --- Serde Helper for serializing/deserializing url::Url ---
// This module provides custom serde logic for `Url` types, as `url::Url` itself
// might not directly implement `Serialize`/`Deserialize` in a way that's always
// compatible with `serde_json` or specific DTO requirements.
// It's kept accessible at the module level for use by `serde(with = "...")`.
mod url_serde_helper {

	use serde::{self, Deserialize, Deserializer, Serializer};
	use url::Url;

	/// Serializes a `&url::Url` into its string representation.
	pub fn serialize<S>(url:&Url, serializer:S) -> Result<S::Ok, S::Error>
	where
		S: Serializer, {
		serializer.serialize_str(url.as_str())
	}

	/// Deserializes a string into a `url::Url`.
	pub fn deserialize<'de, D>(deserializer:D) -> Result<Url, D::Error>
	where
		D: Deserializer<'de>, {
		let s = String::deserialize(deserializer)?;
		Url::parse(&s).map_err(serde::de::Error::custom)
	}
}
