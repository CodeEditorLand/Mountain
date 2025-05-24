// ---------------------------------------------------------------------------------------------
// Mountain Workspace Handlers (handlers/workspace.rs)
// --------------------------------------------------------------------------------------------
// Handles RPC requests from Cocoon's `workspace-shim.js` related to workspace
// information, state, and file searching. Also includes logic for notifying
// Cocoon when workspace state changes.
//
// Responsibilities:
// - Handling `$getWorkspaceFolders`, `$requestWorkspaceTrust`, `$findFiles` RPC
//   calls.
// - Interacting with `AppState` for workspace data.
// - Using `ignore` and `globset` for `findFiles`.
// - Notifying Cocoon of changes.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (or workspace effects).
// - Accesses `AppState`.
// - Uses `vine::send_notification_to_sidecar`.
// - Uses `ignore`, `globset` crates.
// --------------------------------------------------------------------------------------------

use std::{
	path::{Path, PathBuf},
	sync::{
		Arc,
		Mutex as StdMutex,
		MutexGuard,
		atomic::{AtomicBool, Ordering},
	},
};

use globset::{Error as GlobsetError, Glob, GlobBuilder, GlobMatcher};
use ignore::WalkBuilder;
use log::{debug, error, info, trace, warn};
use serde::Deserialize;
use serde_json::{Value, json};
// Removed State as not directly used
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

use crate::{
	app_state::{AppState, WorkspaceFolderState},

	// Use shared error utilities
	handlers::error_utils,

	vine,
};

// Not directly returned by these
// use Land_Common::errors::CommonError;

// handlers

// --- Helper Structs/Enums ---

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct FindFilesOptions {
	max_results:Option<usize>,

	use_ignore_files:Option<bool>,

	use_global_ignore_files:Option<bool>,

	use_parent_ignore_files:Option<bool>,

	follow_symlinks:Option<bool>,
	// VS Code also has `useExcludeSettings` (true by default) and `sortBy` (none, file, type, mtime etc.)
	// Consider adding `useExcludeSettings` if we have a general exclude mechanism in Mountain config.
}

#[derive(Deserialize, Debug)]
// Allows parsing string OR object into this enum
#[serde(untagged)]
enum GlobParam {
	String(String),

	// base is UriComponents JSON Value
	Pattern { pattern:String, base:Option<Value> },
}

// --- Helper Functions ---

/// Helper to map Mutex lock poisoning errors for workspace state.
fn map_workspace_lock_error_to_str<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>, context:&str) -> String {
	let msg = format!("[Workspace Handler LockErr] Failed to acquire lock on {}: {}", context, e);

	error!("{}", msg);

	error_utils::rpc_error_string(msg, Some("ELOCKED_WORKSPACE"))
}

/// Helper to convert Path to file UriComponents JSON Value.
/// Ensures consistent `$mid: 1` for VS Code compatibility.
fn path_to_uri_components_value(p:&Path) -> Value {
	let uri_str = Url::from_file_path(p).map(|url| url.to_string()).unwrap_or_else(|_| {
		warn!(
			"[Workspace Handler] Failed to create file URL from path: {}. Using lossy string.",
			p.display()
		);

		format!("file:///{}", p.to_string_lossy().replace('\\', "/"))
	});

	json!({
		// Important for VS Code URI marshalling
		"$mid": 1,

		"scheme": "file",

		// Path as string
		"path": p.to_str().unwrap_or(""),

		// Full URI string
		"external": uri_str,

		// OS-specific path, often included by VS Code
		"fsPath": p.to_str().unwrap_or("")
	})
}

// --- Request Handlers (Called by Track dispatcher or rpc.rs) ---

pub async fn handle_get_workspace_folders<R:Runtime>(app:AppHandle<R>) -> Result<Value, String> {
	info!("[Workspace Handler] Handling getWorkspaceFolders request");

	let app_state = app.state::<AppState>();

	let folders_lock = app_state
		.workspace_folders
		.lock()
		.map_err(|e| map_workspace_lock_error_to_str(e, "workspace_folders"))?;

	let folders_json:Vec<Value> = folders_lock
		.iter()
		.map(|folder:&WorkspaceFolderState| {
			// Construct the URI components object for the folder's URI
			let folder_uri_components = json!({


				"$mid": 1,

				"scheme": folder.uri.scheme(),

				"authority": folder.uri.host_str().unwrap_or(""),

				"path": folder.uri.path(),

				// Include query if present
				"query": folder.uri.query(),

				// Include fragment if present
				"fragment": folder.uri.fragment(),

				"external": folder.uri.to_string(),

				// Attempt to get fsPath, fallback to path if it's not a file URI or conversion fails
				"fsPath": folder.uri.to_file_path().ok().as_ref().map_or_else(
					// Fallback for non-file URIs or conversion errors
					|| folder.uri.path(),

					|p| p.to_str().unwrap_or("")
				),

			});

			json!({


				// Nest uri components under a "uri" key
				"uri": folder_uri_components,

				"name": folder.name,

				"index": folder.index,

			})
		})
		.collect();

	// Release lock
	drop(folders_lock);

	Ok(json!(folders_json))
}

pub async fn handle_get_workspace_folder<R:Runtime>(
	// Not used in this stub
	_app:AppHandle<R>,

	// Expects Value::Array([uriComponentsToMatch]) or just uriComponentsToMatch
	params_val:Value,
) -> Result<Value, String> {
	let uri_components_to_match = params_val.as_array().and_then(|a| a.get(0)).unwrap_or(params_val);

	warn!(
		"[Workspace Handler] Handling getWorkspaceFolder request (STUBBED): {:?}",
		uri_components_to_match.get("external")
	);

	// TODO:
	// 1. Parse `uri_components_to_match` into a `Url`.
	// 2. Access `app.state::<AppState>().workspace_folders`.
	// 3. Iterate through the folders and find one whose `folder.uri` is an ancestor
	//    of or equal to the target URI. VS Code's
	//    `IExtHostFileSystemInfo#isEqualOrParent` logic is relevant here.
	// 4. If found, serialize that `WorkspaceFolderState` similar to
	//    `handle_get_workspace_folders` and return it.
	// 5. Otherwise, return `Value::Null`.
	Err(error_utils::rpc_error_string(
		"getWorkspaceFolder not fully implemented".to_string(),
		Some("ENOSYS"),
	))
}

pub async fn handle_request_trust<R:Runtime>(
	app:AppHandle<R>,

	// Params might contain details about the trust request in future
	_params:Value,
) -> Result<Value, String> {
	info!("[Workspace Handler] Handling requestWorkspaceTrust request");

	let app_state = app.state::<AppState>();

	let is_trusted = app_state.is_trusted.load(Ordering::Relaxed);

	debug!("[Workspace Handler] Current trust state: {}", is_trusted);

	// For MVP, this returns the current state. A full impl might show a dialog
	// and then call `notify_cocoon_of_trust_change`.
	Ok(json!(is_trusted))
}

pub async fn handle_find_files<R:Runtime>(app_handle:AppHandle<R>, params:Value) -> Result<Value, String> {
	let params_array = params.as_array().ok_or_else(|| {
		error_utils::rpc_param_error_string("findFiles", "params", "array of [include, exclude?, options?]", None)
	})?;

	let include_param_val = params_array.get(0).cloned().ok_or_else(|| {
		error_utils::rpc_param_error_string("findFiles", "include pattern", "GlobParam (string or object)", Some(0))
	})?;

	// Optional
	let exclude_param_val = params_array.get(1).cloned();

	// Optional
	let options_val = params_array.get(2).cloned();

	let include_param:GlobParam = serde_json::from_value(include_param_val).map_err(|e| {
		error_utils::rpc_error_string(format!("Invalid 'include' pattern parameter: {}", e), Some("EBADARG_INCLUDE"))
	})?;

	// Treat null as None
	let exclude_param_opt:Option<GlobParam> = exclude_param_val.filter(|v| !v.is_null())
        // Convert Option<Result> to Result<Option>
		.map(serde_json::from_value).transpose()
		.map_err(|e| error_utils::rpc_error_string(format!("Invalid 'exclude' pattern parameter: {}", e), Some("EBADARG_EXCLUDE")))?;

	let options:FindFilesOptions = options_val
		.map(serde_json::from_value)
		.transpose()
		.map_err(|e| {
			error_utils::rpc_error_string(format!("Invalid 'options' object: {}", e), Some("EBADARG_OPTIONS"))
		})?
		.unwrap_or_default();

	info!(
		"[Workspace Handler] findFiles: include={:?}, exclude={:?}, options={:?}",
		include_param, exclude_param_opt, options
	);

	let app_state = app_handle.state::<AppState>();

	let folders_guard = app_state
		.workspace_folders
		.lock()
		.map_err(|e| map_workspace_lock_error_to_str(e, "workspace_folders for findFiles"))?;

	if folders_guard.is_empty() {
		info!("[Workspace Handler] findFiles: No workspace folders open. Returning empty result.");

		// No folders -> no results
		return Ok(json!([]));
	}

	// Helper to build GlobMatcher, resolving base paths for globs.
	// The returned PathBuf is the effective root for walking if a base was used,

	// otherwise the folder_root.
	let build_matcher = |param:&GlobParam, current_folder_root:&Path| -> Result<(GlobMatcher, PathBuf), String> {
		let (pattern_str, base_uri_components_opt) = match param {
			GlobParam::String(s) => (s.as_str(), None),

			GlobParam::Pattern { pattern, base } => (pattern.as_str(), base.as_ref()),
		};

		let mut effective_glob_base_path:PathBuf;

		if let Some(base_val) = base_uri_components_opt {
			// Base URI is provided in the glob parameter itself
			let scheme = base_val.get("scheme").and_then(Value::as_str).unwrap_or("file");

			if scheme != "file" {
				return Err(error_utils::rpc_error_string(
					format!("Glob base URI must be 'file' scheme, got '{}'", scheme),
					Some("EBADARG_BASE"),
				));
			}
			let base_path_str = base_val.get("path").and_then(Value::as_str).ok_or_else(|| {
				error_utils::rpc_error_string(
					"Glob base URI 'path' field missing or not a string".to_string(),
					Some("EBADARG_BASE"),
				)
			})?;

			effective_glob_base_path = PathBuf::from(base_path_str);

			// Ensure this base path is within the current_folder_root for security/scoping
			if !effective_glob_base_path.starts_with(current_folder_root) {
				warn!(
					"[Workspace Handler] Glob base '{}' is outside current folder root '{}'. Using folder root as \
					 base.",
					effective_glob_base_path.display(),
					current_folder_root.display()
				);

				effective_glob_base_path = current_folder_root.to_path_buf();
			}
		} else {
			// No explicit base, use the current workspace folder root
			effective_glob_base_path = current_folder_root.to_path_buf();
		}

		// If pattern_str is absolute, it defines its own base. Otherwise, it's relative
		// to effective_glob_base_path.
		let glob_pattern_to_compile = if Path::new(pattern_str).is_absolute() {
			pattern_str.to_string()
		} else {
			// Join with base. Glob patterns usually use forward slashes.
			// Convert to string ensuring OS-specific separators are handled if necessary by
			// globset, though globset generally expects POSIX-style paths for patterns.
			effective_glob_base_path
				.join(pattern_str.replace('\\', "/"))
				.to_string_lossy()
				.into_owned()
		};

		trace!(
			"[Workspace Handler] Compiling glob: '{}' (original pattern: '{}', effective base for walk: '{}')",
			glob_pattern_to_compile,
			pattern_str,
			effective_glob_base_path.display()
		);

		let glob = GlobBuilder::new(&glob_pattern_to_compile)
            // OS-dependent case sensitivity for paths
			.case_insensitive(cfg!(windows))
            // On Windows, treat `\` as literal if not escaping. `false` means `/` and `\` are separators.
			.literal_separator(cfg!(windows))
            .build()
            .map_err(|e: GlobsetError| error_utils::rpc_error_string(format!("Invalid glob pattern syntax '{}': {}", glob_pattern_to_compile, e), Some("EBADGLOB_SYNTAX")))?;

		// The path returned is the one WalkBuilder should use as root for this glob
		Ok((glob.compile_matcher(), effective_glob_base_path))
	};

	let mut results:Vec<Value> = Vec::new();

	let max_results = options.max_results.unwrap_or(usize::MAX);

	for folder in folders_guard.iter() {
		if results.len() >= max_results {
			break;

			// Check before processing next folder
		}
		if folder.uri.scheme() != "file" {
			warn!("[Workspace Handler] findFiles: Skipping non-file scheme folder: {}", folder.uri);

			continue;
		}
		let folder_root = PathBuf::from(folder.uri.path());

		debug!("[Workspace Handler] Searching in folder: {}", folder_root.display());

		// Build matchers relative to the current folder_root for this iteration
		let (current_include_matcher, walk_root_for_include) = build_matcher(&include_param, Some(&folder_root))?;

		let current_exclude_matcher_opt = exclude_param_opt.as_ref()
            // Exclude base usually same as include for this setup
			.map(|ex_param| build_matcher(ex_param, Some(&folder_root)).map(|(m, _)| m))
            .transpose()?;

		// Walk from the determined root for this include glob
		let mut walker_builder = WalkBuilder::new(walk_root_for_include.clone());

		// Respect .gitignore etc.
		walker_builder.standard_filters(options.use_ignore_files.unwrap_or(true));

		if options.use_global_ignore_files.unwrap_or(true) {
			// VSCode default is often true for this
			walker_builder.git_global(true);
		}
		walker_builder.follow_links(options.follow_symlinks.unwrap_or(false));

		if let Some(use_parent_ignore) = options.use_parent_ignore_files {
			walker_builder.parents(use_parent_ignore);
		}
		// TODO: Add option `useExcludeSettings` (from VSCode) to respect files.exclude
		// from settings.

		for result_entry in walker_builder.build() {
			if results.len() >= max_results {
				break;
			}
			match result_entry {
				Ok(entry) => {
					let absolute_path = entry.path();

					// The glob patterns compiled by `build_matcher` are effectively absolute or
					// rooted. So, we match them against the absolute path of the entry.
					if current_include_matcher.is_match(absolute_path) {
						if current_exclude_matcher_opt
							.as_ref()
							.map_or(false, |ex| ex.is_match(absolute_path))
						{
							trace!("[Workspace Handler] Excluded by exclude pattern: {}", absolute_path.display());

							continue;
						}
						results.push(path_to_uri_components_value(absolute_path));
					}
				},

				Err(e) => {
					error!(
						"[Workspace Handler] findFiles: Error during directory walk in {}: {}",
						walk_root_for_include.display(),
						e
					)
				},
			}
		}
	}
	// Release lock
	drop(folders_guard);

	info!("[Workspace Handler] findFiles complete. Found {} results.", results.len());

	Ok(json!(results))
}

// --- Notification Helpers (Called internally by Mountain) ---
pub async fn notify_cocoon_of_folder_change<R:Runtime>(app:AppHandle<R>) {
	info!("[Workspace Handler] Notifying Cocoon of workspace folder change");

	// Payload for $onDidChangeWorkspaceFolders is IWorkspaceFoldersChangeEventDto
	// We can send an empty event, or compute added/removed if that info is readily
	// available. For MVP, empty event signals Cocoon to re-request.
	// Minimal event
	let event_payload = json!({ "added": [], "removed": [] });

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$onDidChangeWorkspaceFolders".to_string(), event_payload)
			.await
	{
		error!("[Workspace Handler] Failed to send folder change notification: {}", e);
	}
}
pub async fn notify_cocoon_of_trust_change<R:Runtime>(app:AppHandle<R>, _is_trusted:bool) {
	info!("[Workspace Handler] Notifying Cocoon of workspace trust change");

	// Payload for $onDidGrantWorkspaceTrust is void (empty)
	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$onDidGrantWorkspaceTrust".to_string(), json!({})).await
	{
		error!("[Workspace Handler] Failed to send trust change notification: {}", e);
	}
}
