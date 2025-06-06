// ---------------------------------------------------------------------------------------------
// Mountain Workspace Handlers 
// --------------------------------------------------------------------------------------------
// Handles RPC requests from Cocoon's `workspace-shim.js` related to workspace
// information, state (like trust), and file searching capabilities
// (`vscode.workspace.findFiles`). It also includes logic for notifying Cocoon
// when workspace state (e.g., folders, trust) changes within Mountain.
//
// Responsibilities:
// - Handling RPC calls:
//   - `$getWorkspaceFolders`: Retrieves information about the currently open
//     workspace folders.
//   - `$resolveWorkspaceFolder`: (Stubbed) Intended to find the workspace
//     folder containing a given URI.
//   - `$requestWorkspaceTrust`: (Partially stubbed) Returns current trust
//     state; a full implementation would involve UI interaction.
//   - `$findFiles`: Implements file searching within workspace folders,
//     respecting include/exclude glob patterns and `.gitignore`-style ignore
//     files.
// - Interacting with `AppState` to access workspace data (folders, config path,
//   trust state).
// - Using the `ignore` crate for directory walking and `.gitignore` processing
//   in `$findFiles`.
// - Using the `globset` crate for compiling and matching glob patterns in
//   `$findFiles`.
// - Providing notification helpers (`notify_cocoon_of_folder_change`,
//   `notify_cocoon_of_trust_change`) to send updates to Cocoon via Vine when
//   workspace state changes in Mountain.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (for direct RPCs like
//   `$findFiles`) or by effects created in `track.rs` (for operations like
//   `$getWorkspaceFolders`, `$requestWorkspaceTrust`).
// - Accesses `AppState` for workspace-related information.
// - Uses `vine::send_notification_to_sidecar` for sending notifications to
//   Cocoon.
// - Utilizes `ignore` and `globset` crates for `findFiles` implementation.
// - Uses `handlers::error_utils` for consistent RPC error formatting.
// --------------------------------------------------------------------------------------------

use std::{
	path::{Path, PathBuf},

	sync::{
		MutexGuard,

		// For is_trusted
		atomic::Ordering as AtomicOrdering,
	},
};

use globset::{Error as GlobsetError, GlobBuilder, GlobMatcher};
// For directory walking with ignore file support
use ignore::WalkBuilder;
use log::{debug, error, info, trace, warn};
// For deserializing FindFilesOptions and GlobParam
use serde::Deserialize;
use serde_json::{Value, json};
// `State` from Tauri not directly used in handler signatures here.
use tauri::{AppHandle, Manager, Runtime};
// For handling URIs
use url::Url;

use crate::{
	app_state::{AppState, WorkspaceFolderState},

	// Shared error utilities
	handlers::error_utils,

	// For sending notifications to Cocoon
	vine,
};

// CommonError is not directly returned by these RPC handlers; they return
// String. However, internal operations might map to/from CommonError.
// use Land_Common::errors::CommonError;

// --- Helper Structs/Enums ---

/// Options for the `findFiles` operation, mirroring parts of
/// `vscode.FindFilesOptions`.
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct FindFilesOptions {
	max_results:Option<usize>,

	// .gitignore, .ignore
	use_ignore_files:Option<bool>,

	// Global git ignore
	use_global_ignore_files:Option<bool>,

	// Ignore files in parent dirs
	use_parent_ignore_files:Option<bool>,

	follow_symlinks:Option<bool>,
	// TODO: Add `useExcludeSettings: Option<bool>` to respect `files.exclude` from Mountain's config.
	// TODO: Add `sortBy: Option<String>` if sorting of results is needed.
}

/// Represents a glob pattern parameter for `findFiles`, which can be either a
/// simple string or an object with a pattern and an optional base URI.
/// Mirrors `vscode.GlobPattern`.
#[derive(Deserialize, Debug)]
// Allows deserializing from string OR object
#[serde(untagged)]
enum GlobParam {
	// A simple glob string
	String(String),

	Pattern {
		// The glob pattern string
		pattern:String,

		// Optional base URI (as UriComponents JSON Value)
		base:Option<Value>,
	},
}

// --- Helper Functions ---

/// Formats a `PoisonError` from a Mutex lock on workspace-related `AppState`
/// sections into a standardized RPC error string.
///
/// # Argument
/// * `e` - The `PoisonError`.
/// * `context` - A string describing the context of the lock (e.g.,
///
///
///
///   "workspace_folders").
///
/// # Returns
/// A `String` containing a JSON-formatted RPC error.
fn format_workspace_app_state_lock_error_for_rpc<T>(
	e:std::sync::PoisonError<MutexGuard<'_, T>>,

	context:&str,
) -> String {
	let msg = format!("[Workspace Handler LockErr] Failed to acquire lock on {}: {}", context, e);

	// Log detailed internal error
	error!("{}", msg);

	// Specific lock error code
	error_utils::rpc_error_string(msg, Some("ELOCKED_WORKSPACE"))
}

/// Converts a `Path` to a `serde_json::Value` representing `UriComponents` DTO,
///
///
///
/// specifically for file URIs.
///
/// Ensures consistent inclusion of `$mid: 1` for VS Code compatibility.
///
/// # Argument
/// * `p` - The `Path` to convert.
///
/// # Returns
/// A `serde_json::Value` object for the file URI.
fn file_path_to_uri_components_dto(p:&Path) -> Value {
	let uri_result = Url::from_file_path(p);

	let (uri_scheme, uri_path_str, external_uri_str, fs_path_str) = match uri_result {
		Ok(url) => {
			(
				url.scheme().to_string(),
				url.path().to_string(),
				url.to_string(),
				p.to_string_lossy().into_owned(),
			)
		},

		Err(_) => {
			// Fallback if Path -> Url conversion fails (e.g., invalid chars for URL)
			warn!(
				"[Workspace Helper] Failed to create a valid file URL from path: {}. Using lossy string and 'file:' \
				 scheme.",
				p.display()
			);

			(
				"file".to_string(),
				// Use path as is for path component
				p.to_string_lossy().into_owned(),
				// Attempt basic external form
				format!("file:///{}", p.to_string_lossy().replace('\\', "/")),
				p.to_string_lossy().into_owned(),
			)
		},
	};

	json!({
		// Standard marker for VS Code DTOs needing revival
		"$mid": 1,

		"scheme": uri_scheme,

		// Percent-encoded path from URL object
		"path": uri_path_str,

		// Full URI string
		"external": external_uri_str,

		// OS-specific filesystem path
		"fsPath": fs_path_str
	})
}

// --- Request Handlers  ---

/// Handles the `$getWorkspaceFolders` RPC request from Cocoon.
///
/// Retrieves information about all currently open workspace folders from
/// `AppState` and returns them as an array of `WorkspaceFolder` DTOs.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
///
/// # Returns
/// * `Ok(Value::Array)` where each element is a JSON object representing a
///   workspace folder (`{ uri: UriComponents, name: string, index: number }`).
/// * `Err(String)` with a JSON-RPC error if state access fails.
pub async fn handle_get_workspace_folders<R:Runtime>(app:AppHandle<R>) -> Result<Value, String> {
	info!("[Workspace Handler] Handling $getWorkspaceFolders request");

	let app_state = app.state::<AppState>();

	let folders_guard = app_state
		.workspace_folders
		.lock()
		.map_err(|e| format_workspace_app_state_lock_error_for_rpc(e, "workspace_folders for getWorkspaceFolders"))?;

	let folders_dto_vec:Vec<Value> = folders_guard
		.iter()
		.map(|folder_state:&WorkspaceFolderState| {
			// Construct the UriComponents DTO for the folder's URI
			let folder_uri_components_dto = json!({
				"$mid": 1,

				"scheme": folder_state.uri.scheme(),

				"authority": folder_state.uri.host_str().unwrap_or(""),

				"path": folder_state.uri.path(),

				"query": folder_state.uri.query().map(String::from),

				"fragment": folder_state.uri.fragment().map(String::from),

				"external": folder_state.uri.to_string(),

				"fsPath": folder_state.uri.to_file_path().ok().as_ref().map_or_else(
					// Fallback for non-file URIs
					|| folder_state.uri.path(),

					|p| &p.to_string_lossy().into_owned()
				),

			});

			json!({
				"uri": folder_uri_components_dto,

				"name": folder_state.name,

				"index": folder_state.index,

			})
		})
		.collect();

	// Mutex guard is dropped here.
	drop(folders_guard);

	Ok(json!(folders_dto_vec))
}

/// Handles the `$getWorkspaceFolder` (or `$resolveWorkspaceFolder`) RPC request
/// from Cocoon.
///
/// **STUBBED:** This function is a placeholder. A full implementation would
/// find the workspace folder that contains the given URI.
///
/// # Argument
/// * `_app` - The Tauri `AppHandle` (unused in stub).
/// * `params_val` - A `serde_json::Value` which is expected to be either the
///   `UriComponents` DTO directly, or an array containing it as the first
///   element.
///
/// # Returns
/// * `Err(String)` indicating the function is not fully implemented.
pub async fn handle_get_workspace_folder_for_uri<R:Runtime>(
	// Unused in this stub
	_app:AppHandle<R>,

	// Expects Value::Array([uriComponentsToMatch]) or just uriComponentsToMatch
	params_val:Value,
) -> Result<Value, String> {
	// Extract the UriComponents DTO to match against.
	let uri_components_to_match = params_val
		.as_array()
		// If params_val is an array, take the first element
		.and_then(|a| a.get(0))
		// Otherwise, use params_val directly
		.unwrap_or(&params_val);

	warn!(
		"[Workspace Handler] Handling getWorkspaceFolder/resolveWorkspaceFolder request (STUBBED). Target \
		 URI(external): {:?}",
		uri_components_to_match.get("external")
	);

	// TODO: Implement full logic for $getWorkspaceFolder / $resolveWorkspaceFolder:
	// 1. Parse `uri_components_to_match` into a `url::Url`.
	// 2. Access `app.state::<AppState>().workspace_folders`.
	// 3. Iterate through the `WorkspaceFolderState` entries. For each folder: a.
	//    Convert `folder.uri` to a canonical path if it's a file URI. b. Convert
	//    the target URI to a canonical path if it's a file URI. c. Check if the
	//    target URI's scheme matches the folder's scheme. d. Check if the target
	//    URI's path is a subpath of or equal to the folder's path (similar to VS
	//    Code's `IExtHostFileSystemInfo#isEqualOrParent` logic).
	// 4. If a containing folder is found, serialize that `WorkspaceFolderState`
	//    (similar to `handle_get_workspace_folders`) and return
	//    `Ok(json!(folder_dto))`.
	// 5. If no containing folder is found, return `Ok(Value::Null)`.
	Err(error_utils::rpc_error_string(
		"getWorkspaceFolder / resolveWorkspaceFolder is not fully implemented.".to_string(),
		Some("ENOSYS_WS_RESOLVE"),
	))
}

/// Handles the `$requestWorkspaceTrust` RPC request from Cocoon.
///
/// In this MVP implementation, it returns the current workspace trust state
/// from `AppState`. A full implementation would typically involve showing a UI
/// dialog to the user to grant or deny trust if the state is currently
/// untrusted, and then updating the state and notifying Cocoon.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `_params` - `serde_json::Value` (currently unused, might contain trust
///   request details in the future).
///
/// # Returns
/// * `Ok(Value::Bool)` representing the current trust state.
/// * `Err(String)` if state access fails.
pub async fn handle_request_workspace_trust<R:Runtime>(
	app:AppHandle<R>,

	// Params might contain details about the trust request in future
	_params:Value,
) -> Result<Value, String> {
	info!("[Workspace Handler] Handling $requestWorkspaceTrust request");

	let app_state = app.state::<AppState>();

	let is_trusted = app_state.is_trusted.load(AtomicOrdering::Relaxed);

	debug!(
		"[Workspace Handler] Current workspace trust state: {}. Returning this value for $requestWorkspaceTrust.",
		is_trusted
	);

	// TODO: For a full implementation of $requestWorkspaceTrust:
	// 1. If already trusted, return `Ok(json!(true))`.
	// 2. If not trusted, use `UiProvider` effect to show a trust dialog to the
	//    user. `ui_effects::show_message` with custom buttons ("Grant Trust",

	//    "Later") could work.
	// 3. Based on user's choice: a. If trust granted: Update `app_state.is_trusted`
	//    to `true`. Persist this choice (e.g., in global memento or a dedicated
	//    trust store). Call `notify_cocoon_of_trust_change(app_handle,

	//    true).await`. Return `Ok(json!(true))`. b. If denied or "Later": Return
	//    `Ok(json!(false))`.
	// For MVP, simply returns the current state. The effect in `track.rs` does the
	// same.
	Ok(json!(is_trusted))
}

/// Handles the `$findFiles` RPC request from Cocoon.
///
/// Implements file searching within the workspace folders based on
/// include/exclude glob patterns and options.
///
/// # Argument
/// * `app_handle` - The Tauri `AppHandle`.
/// * `params` - A `serde_json::Value` array: `[include: GlobParam, exclude?:
///   GlobParam | null, options?: FindFilesOptions | null]`
///
/// # Returns
/// * `Ok(Value::Array)` of `UriComponents` DTOs for matching files.
/// * `Err(String)` with a JSON-RPC error if parameters are invalid or search
///   fails.
pub async fn handle_find_files<R:Runtime>(
	app_handle:AppHandle<R>,

	// Array: [include, exclude?, options?]
	params:Value,
) -> Result<Value, String> {
	let params_array = params.as_array().ok_or_else(|| {
		error_utils::rpc_param_error_string(
			"$findFiles",
			"params argument",
			"array of [include, exclude?, options?]",
			None,
		)
	})?;

	let include_param_val = params_array.get(0).cloned().ok_or_else(|| {
		error_utils::rpc_param_error_string(
			"$findFiles",
			"include pattern",
			"GlobParam (string or object {pattern, base?})",
			Some(0),
		)
	})?;

	// Optional
	let exclude_param_val = params_array.get(1).cloned();

	// Optional
	let options_val = params_array.get(2).cloned();

	// Deserialize parameters
	let include_glob_param:GlobParam = serde_json::from_value(include_param_val.clone()).map_err(|e| {
		error_utils::rpc_error_string(
			format!("Invalid 'include' pattern parameter for $findFiles: {}", e),
			Some("EBADARG_INCLUDE_GLOB"),
		)
	})?;

	let exclude_glob_param_opt:Option<GlobParam> = exclude_param_val
		// Treat explicit null as None
		.filter(|v| !v.is_null())
		.map(serde_json::from_value)
		// Convert Option<Result<T, E>> to Result<Option<T>, E>
		.transpose()
		.map_err(|e| {
			error_utils::rpc_error_string(
				format!("Invalid 'exclude' pattern parameter for $findFiles: {}", e),

				Some("EBADARG_EXCLUDE_GLOB"),

			)
		})?;

	let find_options:FindFilesOptions = options_val
		.map(serde_json::from_value)
		.transpose()
		.map_err(|e| {
			error_utils::rpc_error_string(
				format!("Invalid 'options' object for $findFiles: {}", e),

				Some("EBADARG_OPTIONS"),

			)
		})?
		// Use default options if not provided
		.unwrap_or_default();

	info!(
		"[Workspace Handler FindFiles] Request: include={:?}, exclude={:?}, options={:?}",
		include_glob_param, exclude_glob_param_opt, find_options
	);

	let app_state = app_handle.state::<AppState>();

	let folders_guard = app_state
		.workspace_folders
		.lock()
		.map_err(|e| format_workspace_app_state_lock_error_for_rpc(e, "workspace_folders for findFiles"))?;

	if folders_guard.is_empty() {
		info!("[Workspace Handler FindFiles] No workspace folders open. Returning empty result.");

		// No folders means no results
		return Ok(json!([]));
	}

	// Helper to build a GlobMatcher and determine the effective walking root for
	// that glob. `current_folder_root_for_relative_globs` is the root of the
	// workspace folder being iterated.
	let build_glob_matcher_and_walk_root = |glob_param:&GlobParam,

	                                        current_folder_root_for_relative_globs:&Path|
	 -> Result<(GlobMatcher, PathBuf), String> {
		let (pattern_str, base_uri_components_opt) = match glob_param {
			GlobParam::String(s) => (s.as_str(), None),

			GlobParam::Pattern { pattern, base } => (pattern.as_str(), base.as_ref()),
		};

		let mut effective_walk_root_for_this_glob:PathBuf;

		if let Some(base_val) = base_uri_components_opt {
			// Base URI is provided in the GlobParam object.
			let scheme = base_val.get("scheme").and_then(Value::as_str).unwrap_or("file");

			if scheme != "file" {
				return Err(error_utils::rpc_error_string(
					format!("Glob base URI must use 'file' scheme, but got '{}'", scheme),
					Some("EBADARG_GLOB_BASE_SCHEME"),
				));
			}

			let base_path_str_from_dto = base_val.get("path").and_then(Value::as_str).ok_or_else(|| {
				error_utils::rpc_error_string(
					"Glob base URI 'path' field missing or not a string".to_string(),
					Some("EBADARG_GLOB_BASE_PATH"),
				)
			})?;

			effective_walk_root_for_this_glob = PathBuf::from(base_path_str_from_dto);

			// Security/Scoping: Ensure the provided base path is within the current
			// workspace folder. TODO: Canonicalize paths for robust checking.
			if !effective_walk_root_for_this_glob.starts_with(current_folder_root_for_relative_globs) {
				warn!(
					"[Workspace Handler FindFiles] Glob base '{}' is outside current folder root '{}'. Clamping walk \
					 root to folder root.",
					effective_walk_root_for_this_glob.display(),
					current_folder_root_for_relative_globs.display()
				);

				effective_walk_root_for_this_glob = current_folder_root_for_relative_globs.to_path_buf();
			}
		} else {
			// No explicit base in GlobParam, use the current workspace folder root as the
			// base for walking.
			effective_walk_root_for_this_glob = current_folder_root_for_relative_globs.to_path_buf();
		}

		// Construct the glob pattern string to compile.
		// If `pattern_str` is absolute, it defines its own context.
		// Otherwise, it's relative to `effective_walk_root_for_this_glob`.
		// `globset` expects POSIX-style paths for patterns.
		let final_glob_pattern_to_compile = if Path::new(pattern_str).is_absolute() {
			// Normalize to POSIX separators
			pattern_str.replace('\\', "/")
		} else {
			// Join relative pattern with its effective base path.
			effective_walk_root_for_this_glob
				// `pattern_str` should ideally be POSIX style already
				.join(pattern_str)
				.to_string_lossy()
				// Ensure POSIX separators
				.replace('\\', "/")
		};

		trace!(
			"[Workspace Handler FindFiles] Compiling glob: '{}' (original pattern: '{}', effective walk root: '{}')",
			final_glob_pattern_to_compile,
			pattern_str,
			effective_walk_root_for_this_glob.display()
		);

		let glob = GlobBuilder::new(&final_glob_pattern_to_compile)
			// OS-dependent case sensitivity for paths
			.case_insensitive(cfg!(windows))
			// On Windows, `\` is a literal if true; false means `\` is separator
			.literal_separator(cfg!(windows))
			.build()
			.map_err(|e: GlobsetError| {
				error_utils::rpc_error_string(
					format!("Invalid glob pattern syntax '{}': {}", final_glob_pattern_to_compile, e),

					Some("EBADGLOB_SYNTAX"),

				)
			})?;

		Ok((glob.compile_matcher(), effective_walk_root_for_this_glob))
	};

	let mut results_uri_dtos:Vec<Value> = Vec::new();

	let max_results_cap = find_options.max_results.unwrap_or(usize::MAX);

	for folder_state in folders_guard.iter() {
		if results_uri_dtos.len() >= max_results_cap {
			// Stop if max results reached
			break;
		}

		if folder_state.uri.scheme() != "file" {
			warn!(
				"[Workspace Handler FindFiles] Skipping search in non-file scheme folder: {}",
				folder_state.uri
			);

			continue;
		}

		let current_folder_root_path = PathBuf::from(folder_state.uri.path());

		debug!(
			"[Workspace Handler FindFiles] Searching in folder: {}",
			current_folder_root_path.display()
		);

		// Build include matcher and determine its walk root for this folder.
		let (current_include_matcher, walk_root_for_include) =
			build_glob_matcher_and_walk_root(&include_glob_param, &current_folder_root_path)?;

		// Build exclude matcher (if any) for this folder.
		let current_exclude_matcher_opt = exclude_glob_param_opt
			.as_ref()
			.map(|ex_param| build_glob_matcher_and_walk_root(ex_param, &current_folder_root_path).map(|(m, _)| m))
			// Converts Option<Result<T,E>> to Result<Option<T>,E>
			.transpose()?;

		// Configure WalkBuilder for directory traversal.
		let mut walker_builder = WalkBuilder::new(&walk_root_for_include);

		// Respect .gitignore etc.
		walker_builder.standard_filters(find_options.use_ignore_files.unwrap_or(true));

		walker_builder.git_global(find_options.use_global_ignore_files.unwrap_or(true));

		walker_builder.follow_links(find_options.follow_symlinks.unwrap_or(false));

		if let Some(use_parent_ignore) = find_options.use_parent_ignore_files {
			walker_builder.parents(use_parent_ignore);
		}

		// TODO: Implement `useExcludeSettings` by adding an `ignore::Override` based on
		// `files.exclude` settings.
		for result_entry_res in walker_builder.build() {
			if results_uri_dtos.len() >= max_results_cap {
				// Check max results within the loop too
				break;
			}

			match result_entry_res {
				Ok(dir_entry) => {
					let absolute_entry_path = dir_entry.path();

					// Glob patterns compiled by `build_glob_matcher_and_walk_root` are absolute or
					// effectively absolute (rooted at their walk_root). Match against the
					// absolute path of the entry.
					if current_include_matcher.is_match(absolute_entry_path) {
						if current_exclude_matcher_opt
							.as_ref()
							.map_or(false, |ex_matcher| ex_matcher.is_match(absolute_entry_path))
						{
							trace!(
								"[Workspace Handler FindFiles] Path excluded by exclude pattern: {}",
								absolute_entry_path.display()
							);

							// Skip if matches exclude pattern
							continue;
						}

						results_uri_dtos.push(file_path_to_uri_components_dto(absolute_entry_path));
					}
				},

				Err(e) => {
					// Log errors during walk (e.g., permission denied for a subdir) but continue.
					error!(
						"[Workspace Handler FindFiles] Error during directory walk in '{}': {}",
						walk_root_for_include.display(),
						e
					);
				},
			}
		}
	}

	// Release lock on workspace_folders
	drop(folders_guard);

	info!(
		"[Workspace Handler FindFiles] Search complete. Found {} results (cap: {}).",
		results_uri_dtos.len(),
		max_results_cap
	);

	Ok(json!(results_uri_dtos))
}

// --- Notification Helpers (Called internally by Mountain to inform Cocoon) ---

/// Notifies Cocoon that the set of workspace folders has changed.
///
/// Cocoon, upon receiving this, will typically re-request the workspace folders
/// via `$getWorkspaceFolders`.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
pub async fn notify_cocoon_of_workspace_folder_change<R:Runtime>(_:AppHandle<R>) {
	info!("[Workspace Handler Notify] Notifying Cocoon of workspace folder change via $onDidChangeWorkspaceFolders");

	// Payload for $onDidChangeWorkspaceFolders is IWorkspaceFoldersChangeEventDto:
	// { added: IWorkspaceFolder[], removed: IWorkspaceFolder[], changed:
	// IWorkspaceFolder[] } For MVP, sending an empty event signals Cocoon to
	// re-request all folders. TODO: Provide actual added/removed/changed folder
	// details in the event payload       if this information is readily available
	// when this notification is triggered.
	let event_payload = json!({ "added": [], "removed": [], "changed": [] });

	if let Err(e) = vine::send_notification_to_sidecar(
		// TODO: Make sidecar ID configurable
		"cocoon-main",
		"$onDidChangeWorkspaceFolders".to_string(),
		event_payload,
	)
	.await
	{
		error!(
			"[Workspace Handler Notify] Failed to send $onDidChangeWorkspaceFolders notification to Cocoon: {}",
			e
		);
	}
}

/// Notifies Cocoon that the workspace trust state has changed.
///
/// Cocoon, upon receiving `$onDidGrantWorkspaceTrust`, typically re-evaluates
/// trust-dependent features.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `_is_trusted` - The new trust state (boolean, currently unused in this
///   simple notification).
pub async fn notify_cocoon_of_workspace_trust_change<R:Runtime>(
	_:AppHandle<R>,

	// Parameter kept for future use if payload needs it
	_is_trusted:bool,
) {
	info!("[Workspace Handler Notify] Notifying Cocoon of workspace trust change via $onDidGrantWorkspaceTrust");

	// The payload for `$onDidGrantWorkspaceTrust` is void (empty object or no
	// payload).
	if let Err(e) = vine::send_notification_to_sidecar(
		// TODO: Make sidecar ID configurable
		"cocoon-main",
		"$onDidGrantWorkspaceTrust".to_string(),
		// Empty object payload
		json!({}),
	)
	.await
	{
		error!(
			"[Workspace Handler Notify] Failed to send $onDidGrantWorkspaceTrust notification to Cocoon: {}",
			e
		);
	}
}
