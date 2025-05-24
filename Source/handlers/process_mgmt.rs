// ---------------------------------------------------------------------------------------------
// Mountain Process Management Handlers (handlers/process_mgmt.rs)
// --------------------------------------------------------------------------------------------
// Responsible for launching, managing the lifecycle of, and establishing
// initial communication with sidecar processes, specifically the Cocoon Node.js
// extension host.
//
// Responsibilities:
// - Locating executables/scripts (Node.js, Cocoon/index.js) via Tauri's
//   PathResolver.
// - Spawning the sidecar process asynchronously (`tokio::process::Command`).
// - Configuring stdio pipes for IPC.
// - Initiating `Vine` IPC setup (`vine::setup_sidecar_communication`).
// - Orchestrating the initial handshake:
//   - Waiting for `vine://sidecar/ready` event from Cocoon.
//   - Constructing `IExtensionHostInitData` using `AppState` and system info.
//   - Sending `initData` to Cocoon via `vine::send_notification_to_sidecar`.
// - Monitoring process exit and triggering cleanup via
//   `vine::unregister_sidecar`.
//
// Key Interactions:
// - Called from `main.rs` Tauri setup hook.
// - Uses Tauri `PathResolver` and `AppHandle` for resources and state.
// - Uses `tokio::process::Command` for spawning.
// - Interacts heavily with `Vine` for IPC setup and communication.
// - Listens for Tauri events (e.g., `vine://sidecar/ready`).
// - Accesses `AppState` to construct `IExtensionHostInitData`.
// - Uses `log` for detailed logging of process lifecycle and IPC events.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	path::PathBuf,
	process::Stdio,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

// For mapping log::LevelFilter to VS Code's LogLevel enum in initData
use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Listener, Manager, Runtime};
use tokio::{
	io::{AsyncBufReadExt, BufReader},

	process::Command,

	// Tokio MPSC for internal signaling
	sync::mpsc as tokio_mpsc,

	time::timeout,
};
use url::Url;

use crate::{
	app_state::{AppState, ExtensionDescriptionState, WorkspaceFolderState},

	vine,
	// error_utils is not strictly needed here as public functions don't return Result<_, String>
	// but could be used for internal error formatting if desired.
	// handlers::error_utils,
};

// CommonError is not directly returned by public functions here.
// use Land_Common::errors::CommonError;

// --- Main Launch Function ---

/// Launches and manages the Cocoon (Node.js Extension Host) sidecar process.
///
/// This function is responsible for:
/// 1. Locating the Node.js executable and Cocoon main script bundled with the
///    application.
/// 2. Spawning the Cocoon process with appropriate arguments (e.g., parent
///    PID).
/// 3. Setting up stdio pipes for IPC.
/// 4. Initializing Vine communication with the spawned sidecar.
/// 5. Triggering the sending of `IExtensionHostInitData` once Cocoon signals
///    readiness.
/// 6. Monitoring the sidecar process for exit and performing cleanup.
///
/// This function is typically called once during Mountain's startup.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`, used for path resolution, state
///   access, and event handling.
pub async fn launch_and_manage_cocoon_sidecar<R:Runtime>(app_handle:AppHandle<R>) {
	info!("[ProcMgmt] Attempting to launch Cocoon Sidecar (Node.js Extension Host)...");

	// TODO: Make sidecar ID configurable if multiple sidecars of different types
	// are supported.
	let sidecar_id = "cocoon-main".to_string();

	// --- 1. Determine Paths to Node and Cocoon Script ---
	let path_resolver = app_handle.path_resolver();

	// Path to bundled Node.js
	let node_path_opt = path_resolver.resolve_resource("bin/node");

	// Path to Cocoon's entry script
	let script_path_opt = path_resolver.resolve_resource("scripts/cocoon/index.js");

	let (node_path_str, script_path_str) = match (node_path_opt, script_path_opt) {
		(Some(node_path), Some(script_path)) => {
			if !node_path.exists() || !node_path.is_file() {
				error!(
					"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Bundled Node.js executable not found or is not a file at \
					 resolved path: {}. Cocoon cannot start. Ensure 'bin/node' is correctly bundled (e.g., via Tauri \
					 'resources' or externalBin) and is executable.",
					node_path.display()
				);

				return;
			}

			if !script_path.exists() || !script_path.is_file() {
				error!(
					"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Cocoon main script (index.js) not found or is not a file at \
					 resolved path: {}. Cocoon cannot start. Check Tauri 'resources' configuration.",
					script_path.display()
				);

				return;
			}

			info!("[ProcMgmt] Using bundled Node.js: {}", node_path.display());

			info!("[ProcMgmt] Using Cocoon script: {}", script_path.display());

			(
				node_path.to_string_lossy().into_owned(),
				script_path.to_string_lossy().into_owned(),
			)
		},

		(node_res_opt, script_res_opt) => {
			error!(
				"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Failed to resolve one or more bundled paths for Cocoon. Node \
				 resolved: {}, Script resolved: {}. Ensure 'bin/node' and 'scripts/cocoon/index.js' are correctly \
				 listed in tauri.conf.json's resources/externalBin and are present in the target/release or \
				 target/debug directory.",
				node_res_opt.is_some(),
				script_res_opt.is_some()
			);

			return;
		},
	};

	// --- 2. Prepare Command to Spawn Cocoon ---
	let mut command = Command::new(&node_path_str);

	command.arg(&script_path_str);

	// Pass Mountain's Process ID to Cocoon, useful for Cocoon to monitor its
	// parent.
	command.arg(format!("--parent-pid={}", std::process::id()));

	// TODO: Add other necessary command-line arguments for Cocoon:
	//       - Port for inspector/debugger if enabled.
	//       - Log level for Cocoon.
	//       - Path to extension directories if not passed via initData.

	// Configure stdio for IPC with Vine.
	// For sending messages to Cocoon
	command.stdin(Stdio::piped());

	// For receiving messages from Cocoon
	command.stdout(Stdio::piped());

	// Capture Cocoon's stderr for logging
	command.stderr(Stdio::piped());

	// Ensure child process is killed when the `Command` handle (or `Child` handle)
	// is dropped if Mountain exits unexpectedly.
	command.kill_on_drop(true);

	info!(
		"[ProcMgmt] Spawning Cocoon command: {} {} --parent-pid={}",
		node_path_str,
		script_path_str,
		std::process::id()
	);

	// --- 3. Spawn the Cocoon Process ---
	match command.spawn() {
		Ok(mut child_process) => {
			// Option<u32>
			let child_pid_opt = child_process.id();

			let child_pid_log_str = child_pid_opt.map_or_else(
				|| "unknown (spawned but ID not immediately available)".to_string(),
				|pid| pid.to_string(),
			);

			info!("[ProcMgmt] Cocoon process spawned successfully [OS PID: {}]", child_pid_log_str);

			// --- 3a. Set up Stderr Logging for Cocoon ---
			if let Some(stderr_stream) = child_process.stderr.take() {
				let stderr_sidecar_id_clone = sidecar_id.clone();

				let stderr_pid_for_log_clone = child_pid_log_str.clone();

				tokio::spawn(async move {
					info!(
						"[Cocoon stderr ({})][PID: {}] Stderr monitoring task started.",
						stderr_sidecar_id_clone, stderr_pid_for_log_clone
					);

					let reader = BufReader::new(stderr_stream);

					let mut lines = reader.lines();

					while let Ok(Some(line)) = lines.next_line().await {
						// Log Cocoon's stderr output using Mountain's logger.
						// Prepend with sidecar ID for clarity.
						error!(
							"[Cocoon stderr ({})][PID: {}] {}",
							stderr_sidecar_id_clone, stderr_pid_for_log_clone, line
						);
					}

					info!(
						"[Cocoon stderr ({})][PID: {}] Stderr stream closed.",
						stderr_sidecar_id_clone, stderr_pid_for_log_clone
					);
				});
			} else {
				warn!(
					"[ProcMgmt] Could not capture stderr stream for Cocoon sidecar '{}'. Its stderr output will not \
					 be logged by Mountain.",
					sidecar_id
				);
			}

			// --- 4. Setup Vine IPC ---
			// `vine::setup_sidecar_communication` takes ownership of stdin/stdout from
			// `child_process`.
			match vine::setup_sidecar_communication(
				sidecar_id.clone(),
				// `child_process` is moved here (or parts of it)
				child_process,
				app_handle.clone(),
			) {
				Ok(_) => {
					info!(
						"[ProcMgmt] Vine communication setup initiated for '{}' [OS PID: {}]",
						sidecar_id, child_pid_log_str
					);

					// --- 5. Trigger initData Sending (after Vine setup) ---
					spawn_task_to_send_init_data_after_ready_signal(app_handle.clone(), sidecar_id.clone());
				},

				Err(vine_error) => {
					error!(
						"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Failed to setup Vine IPC for '{}' [OS PID: {}]: {}. \
						 Cocoon sidecar will be non-functional.",
						sidecar_id, child_pid_log_str, vine_error
					);

					// TODO: Consider if Mountain should attempt to kill the
					// child process here       if Vine setup fails, as
					// it's unlikely to be useful.       `child_process.
					// kill().await` could be used, but `child_process` was
					// moved.       `vine::setup_sidecar_communication`
					// would need to return the `Child` on error,

					//       or `kill_on_drop` on the original `Command` handle
					// should take care of it       if this function
					// returns and the `Child` object from `spawn()` goes out of
					// scope.
				},
			}
		},

		Err(spawn_error) => {
			error!(
				"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Failed to spawn Cocoon process using Node path '{}'. Error: {}. \
				 Ensure Node.js is executable, script path is correct, and no permission issues.",
				node_path_str, spawn_error
			);
		},
	}
}

/// Spawns an asynchronous task that waits for the `vine://sidecar/ready` signal
/// from the specified sidecar, then constructs and sends
/// `IExtensionHostInitData`.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`.
/// * `sidecar_id` - The ID of the sidecar to send initData to.
fn spawn_task_to_send_init_data_after_ready_signal<R:Runtime>(app_handle:AppHandle<R>, sidecar_id:String) {
	tokio::spawn(async move {
		info!(
			"[ProcMgmt InitSender] Task for sidecar '{}' waiting for 'vine://sidecar/ready' signal...",
			sidecar_id
		);

		// MPSC channel to signal readiness from the Tauri event listener to this task.
		let (tx_ready_signal, mut rx_ready_signal) = tokio_mpsc::channel::<()>(1);

		let received_signal_flag = Arc::new(AtomicBool::new(false));

		// Clone necessary items for the listener closure.
		let listener_app_handle = app_handle.clone();

		let listener_sidecar_id = sidecar_id.clone();

		let listener_received_flag = received_signal_flag.clone();

		// Listen for the global Tauri event `vine://sidecar/ready`.
		// The payload of this event is expected to be the `sidecar_id` as a string.
		let tauri_event_listener_id = app_handle.listen_global("vine://sidecar/ready", move |event| {
			if let Some(payload_str) = event.payload() {
				match serde_json::from_str::<String>(payload_str) {
					Ok(ready_id_from_event) if ready_id_from_event == listener_sidecar_id => {
						// Atomically set flag and send signal only once.
						if listener_received_flag
							.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
							.is_ok()
						{
							info!(
								"[ProcMgmt InitSender] Received 'vine://sidecar/ready' signal for target sidecar '{}'.",
								listener_sidecar_id
							);

							if tx_ready_signal.try_send(()).is_err() {
								error!(
									"[ProcMgmt InitSender] Failed to send internal ready confirmation for '{}' \
									 (receiver likely dropped due to timeout or task cancellation).",
									listener_sidecar_id
								);
							}
						} else {
							warn!(
								"[ProcMgmt InitSender] Duplicate 'vine://sidecar/ready' signal received for '{}'. \
								 Ignoring.",
								listener_sidecar_id
							);
						}
					},

					Ok(other_id) => {
						// Signal was for a different sidecar.
						trace!(
							"[ProcMgmt InitSender] 'vine://sidecar/ready' signal received, but for different sidecar \
							 '{}' (expecting '{}').",
							other_id, listener_sidecar_id
						);
					},

					Err(e) => {
						error!(
							"[ProcMgmt InitSender] Failed to parse 'vine://sidecar/ready' event payload as string: \
							 '{}'. Payload: {}",
							e, payload_str
						);
					},
				}
			} else {
				warn!(
					"[ProcMgmt InitSender] Received 'vine://sidecar/ready' event with no payload for sidecar '{}'. \
					 Cannot verify target.",
					listener_sidecar_id
				);
			}
		});

		debug!(
			"[ProcMgmt InitSender] Listening for 'vine://sidecar/ready' from '{}' (Tauri listener ID: {}).",
			sidecar_id, tauri_event_listener_id
		);

		// Timeout for waiting for the ready signal.
		// TODO: Make this timeout configurable.
		let ready_timeout_duration = Duration::from_secs(30);

		match timeout(ready_timeout_duration, rx_ready_signal.recv()).await {
			Ok(Some(_)) => {
				info!(
					"[ProcMgmt InitSender] Internal ready confirmation received for '{}'. Proceeding to send initData.",
					sidecar_id
				);
			},

			Ok(None) => {
				// MPSC channel closed before message, tx_ready_signal likely dropped or task
				// cancelled.
				error!(
					"[ProcMgmt InitSender] Internal ready signal channel for '{}' closed unexpectedly. Aborting \
					 initData send. Sidecar may not have signaled readiness or listener was removed.",
					sidecar_id
				);

				app_handle.unlisten(tauri_event_listener_id);

				// Ensure cleanup if init fails at this stage.
				vine::unregister_sidecar(&sidecar_id);

				return;
			},

			Err(_timeout_elapsed_error) => {
				error!(
					"[ProcMgmt InitSender] Timed out ({:?}) waiting for 'vine://sidecar/ready' signal from '{}'. \
					 Sidecar might have failed to initialize or signal. Aborting initData send.",
					ready_timeout_duration, sidecar_id
				);

				app_handle.unlisten(tauri_event_listener_id);

				// Critical to unregister if sidecar is unresponsive.
				vine::unregister_sidecar(&sidecar_id);

				return;
			},
		}

		// Unlisten once the signal has been processed or timed out to prevent resource
		// leaks.
		app_handle.unlisten(tauri_event_listener_id);

		info!(
			"[ProcMgmt InitSender] Constructing and sending 'initExtensionHost' data to '{}'...",
			sidecar_id
		);

		// Construct the detailed initialization payload.
		// This can panic if AppState locks are poisoned, which is a critical failure at
		// startup.
		let init_data_payload = construct_extension_host_init_data(&app_handle);

		trace!(
			"[ProcMgmt InitSender] Constructed initData for '{}': (Top-level keys: {:?})",
			sidecar_id,
			init_data_payload.as_object().map(|o| o.keys().collect::<Vec<_>>())
		);

		// Send the initData as a notification to the sidecar via Vine.
		match vine::send_notification_to_sidecar(&sidecar_id, "initExtensionHost".to_string(), init_data_payload).await
		{
			Ok(_) => {
				info!(
					"[ProcMgmt InitSender] 'initExtensionHost' notification sent successfully to '{}'.",
					sidecar_id
				)
			},

			Err(e) => {
				error!(
					"[ProcMgmt InitSender] Failed to send 'initExtensionHost' notification to '{}': {}. Sidecar may \
					 not initialize correctly.",
					sidecar_id, e
				)
			},
		}
	});
}

/// Constructs the `IExtensionHostInitData` payload required by Cocoon.
///
/// This function gathers information from `AppState`, Tauri's `PathResolver`,
///
/// and system properties to build a comprehensive JSON object that matches the
/// structure expected by VS Code's extension host.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`.
///
/// # Returns
/// A `serde_json::Value` representing the `IExtensionHostInitData`.
///
/// # Panics
/// This function can panic if critical `AppState` locks are poisoned, as this
/// indicates an unrecoverable state during startup.
fn construct_extension_host_init_data<R:Runtime>(app_handle:&AppHandle<R>) -> Value {
	let app_state = app_handle.state::<AppState>();

	info!("[ProcMgmt InitData] Constructing IExtensionHostInitData...");

	// Helper to create UriComponents JSON Value from a PathBuf.
	// Ensures consistent `$mid: 1` for VS Code DTO compatibility.
	let path_to_uri_components_dto = |p:PathBuf, is_dir:bool| -> Value {
		let url_res = if is_dir { Url::from_directory_path(&p) } else { Url::from_file_path(&p) };

		let url = url_res.unwrap_or_else(|_| {
			warn!(
				"[ProcMgmt InitData] Failed to create URL from path: {}. Using lossy string with 'file:' scheme.",
				p.display()
			);

			// Fallback for paths that don't convert cleanly to system-specific file URLs
			Url::parse(&format!("file:///{}", p.to_string_lossy().replace('\\', "/")))
				.expect("Fallback URL parse for initData path failed catastrophically")
		});

		json!({

			"scheme": url.scheme(),

			// Ensure authority is string even if empty
			"authority": url.host_str().unwrap_or(""),

			"path": url.path(),

			// Option<String>
			"query": url.query().map(String::from),

			// Option<String>
			"fragment": url.fragment().map(String::from),

			// Full URI string
			"external": url.to_string(),

			// Standard VS Code DTO marker
			"$mid": 1
		})
	};

	// --- Workspace Data ---
	let (
		workspace_id_str,
		workspace_name_str,
		// Option<Value>
		workspace_config_uri_dto,
		// Vec<Value>
		workspace_folders_dto_vec,
		is_transient_bool,
		is_untitled_bool,
	) = {
		let folders_guard = app_state
			.workspace_folders
			.lock()
			.expect("Lock workspace_folders for initData failed; AppState poisoned.");

		let config_path_guard = app_state
			.workspace_config_path
			.lock()
			.expect("Lock workspace_config_path for initData failed; AppState poisoned.");

		let id = app_state.get_workspace_id_string().unwrap_or_else(|e| {
			warn!(
				"[ProcMgmt InitData] Failed to get workspace ID string: {}. Using default 'FALLBACK_WORKSPACE_ID'.",
				e
			);

			"FALLBACK_WORKSPACE_ID".to_string()
		});

		let name = app_state.get_workspace_name().unwrap_or_else(|e| {
			warn!(
				"[ProcMgmt InitData] Failed to get workspace name: {}. Using default 'Untitled Workspace'.",
				e
			);

			"Untitled Workspace".to_string()
		});

		// Mountain workspaces are not typically transient like in remote development
		// scenarios.
		let transient = false;

		// Determine if workspace is untitled (no config file, <=1 untitled folder).
		let untitled = config_path_guard.is_none()
			&& folders_guard.len() <= 1
			&& (folders_guard.first().map_or(true, |f| f.uri.scheme() == "untitled"));

		let config_components_opt = config_path_guard.as_ref().map(|p| path_to_uri_components_dto(p.clone(), false));

		let folders_components_vec:Vec<Value> = folders_guard
			.iter()
			.map(|f_state:&WorkspaceFolderState| {
				let f_uri_comp_dto = json!({

					"scheme": f_state.uri.scheme(),

					"authority": f_state.uri.host_str().unwrap_or(""),

					"path": f_state.uri.path(),

					"query": f_state.uri.query().map(String::from),

					"fragment": f_state.uri.fragment().map(String::from),

					"external": f_state.uri.to_string(),

					"$mid": 1
				});

				json!({ "uri": f_uri_comp_dto, "name": f_state.name, "index": f_state.index })
			})
			.collect();

		debug!(
			"[ProcMgmt InitData] Workspace: ID='{}', Name='{}', ConfigPathPresent={}, NumFolders={}, Untitled={}",
			id,
			name,
			config_components_opt.is_some(),
			folders_components_vec.len(),
			untitled
		);

		(id, name, config_components_opt, folders_components_vec, transient, untitled)
	};

	// `IStaticWorkspaceData` or `null` if no workspace is effectively open.
	let workspace_data_val =
		if workspace_folders_dto_vec.is_empty() && workspace_config_uri_dto.is_none() && !is_untitled_bool {
			// No workspace if no folders, no config, and not explicitly untitled.
			Value::Null
		} else {
			json!({

				"id": workspace_id_str,

				"name": workspace_name_str,

				// URI to .code-workspace or null
				"configuration": workspace_config_uri_dto.unwrap_or(Value::Null),

				"folders": workspace_folders_dto_vec,

				"transient": is_transient_bool,

				"isUntitled": is_untitled_bool
			})
		};

	// --- Extensions Data (IExtensionDescriptionSnapshot) ---
	let (all_extensions_desc_dto_vec, my_extensions_ids_dto_vec, activation_events_map_dto) = {
		let scanned_extensions_guard = app_state
			.scanned_extensions
			.lock()
			.expect("Lock scanned_extensions for initData failed; AppState poisoned.");

		let mut all_ext_descs:Vec<Value> = Vec::new();

		// IDs of extensions that should run in *this* extension host.
		let mut my_ext_ids:Vec<Value> = Vec::new();

		// Map<extensionIdString, activationEventsArray>
		let mut act_events:HashMap<String, Vec<String>> = HashMap::new();

		for (ext_full_id_str, ext_desc_state) in scanned_extensions_guard.iter() {
			// For MVP, assume all scanned extensions run in this primary (Cocoon) host.
			// TODO: Implement logic for extension host affinity if multiple host types are
			// supported.
			match serde_json::to_value(ext_desc_state.clone()) {
				Ok(serialized_desc_dto) => {
					all_ext_descs.push(serialized_desc_dto);

					// `identifier` is `{value: "pub.name", uuid?: "..."}` DTO
					my_ext_ids.push(ext_desc_state.identifier.clone());

					if let Some(events_vec) = &ext_desc_state.activation_events {
						if !events_vec.is_empty() {
							act_events.insert(ext_full_id_str.clone(), events_vec.clone());
						}
					}
				},

				Err(e) => {
					error!(
						"[ProcMgmt InitData] Failed to serialize ExtensionDescriptionState for '{}': {}",
						ext_full_id_str, e
					);

					// TODO: Consider if this should be a fatal error or if
					// startup can proceed without this extension.
				},
			}
		}

		info!(
			"[ProcMgmt InitData] Processed {} scanned extensions for initData.",
			all_ext_descs.len()
		);

		(all_ext_descs, my_ext_ids, act_events)
	};

	let extensions_snapshot_data_dto = json!({

		// Simple versioning for the snapshot itself, can be incremented if snapshot structure changes.
		"versionId": 1,

		"allExtensions": all_extensions_desc_dto_vec,

		// Extensions specifically for this host instance
		"myExtensions": my_extensions_ids_dto_vec,

		// Map: extId -> activationEvents[]
		"activationEvents": activation_events_map_dto
	});

	// --- Configuration Data (IConfigurationInitData) ---
	let configuration_data_dto = {
		let config_guard = app_state
			.configuration
			.lock()
			.expect("Lock configuration for initData failed; AppState poisoned.");

		// This sends the merged "effective" configuration.
		// TODO: Populate 'defaults', 'user', 'workspace', 'folders' if a more granular
		// multi-file config model is implemented.
		json!({

			"effective": config_guard.data.clone(),

			// Stub
			"defaults": { "contents": {} },

			// Stub
			"user": { "contents": {} },

			// Stub
			"workspace": { "contents": {} },

			// Stub
			"folders": [],

			// Stub
			"memory": { "contents": {} },

			// Stub
			"policy": Value::Null,

			// Stub
			"configurationScopes": []
		})
	};

	trace!(
		"[ProcMgmt InitData] Configuration data DTO effective keys count: {}",
		configuration_data_dto["effective"].as_object().map_or(0, |o| o.keys().len())
	);

	// --- Paths ---
	let path_resolver = app_handle.path_resolver();

	let logs_loc_path = path_resolver
		.app_log_dir()
		// Fallback if system log dir isn't found
		.unwrap_or_else(|| PathBuf::from("./mountain_dev_logs_fallback"));

	let app_data_dir_base = path_resolver
		.app_data_dir()
		.unwrap_or_else(|| PathBuf::from("./mountain_dev_appdata_fallback"));

	// VS Code typically uses `User/globalStorage` and
	// `User/workspaceStorage/<workspace_id_hash>`
	let user_data_root_path = app_data_dir_base.join("User");

	let global_storage_home_path = user_data_root_path.join("globalStorage");

	let ws_storage_home_id_str = app_state
		.get_workspace_id_string()
		.unwrap_or_else(|_| "NO_WORKSPACE_ID_FOR_STORAGE_PATH".to_string());

	let workspace_storage_home_path = user_data_root_path.join("workspaceStorage").join(ws_storage_home_id_str);

	// `appRoot` is the root of the application installation.
	// For Tauri, this might be complex. `app_config_dir` or `app_data_dir`'s parent
	// might be closer. Or, path_resolver.resource_dir() if resources are at top
	// level of install. Using app_config_dir's parent as a heuristic for now.
	let app_root_heuristic_path = path_resolver
		.app_config_dir()
		.and_then(|p| p.parent().map(PathBuf::from))
		.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

	// --- Enabled Proposed APIs ---
	let enabled_proposed_apis_val = {
		let guard = app_state
			.enabled_proposed_apis
			.lock()
			.expect("Lock proposed APIs for initData failed; AppState poisoned.");

		// If "*" is present, its value is an array of API names.
		// Otherwise, the map is extId -> apiNames[].
		// Cocoon expects a map: { [extensionId: string]: string[] }

		// or a flat array if all are enabled for "*" (less common for VS Code).
		// Let's assume Cocoon can handle the map directly.
		json!(*guard)
	};

	debug!(
		"[ProcMgmt InitData] Enabled Proposed APIs for initData: {:?}",
		enabled_proposed_apis_val
	);

	// --- Telemetry Info ---
	// Use Tauri's instance ID for machineId. Session ID is unique per launch.
	let machine_id_str = app_handle.manager().instance_id().to_string();

	let session_id_str = uuid::Uuid::new_v4().to_string();

	// ISO 8601 format
	let first_session_date_str = chrono::Utc::now().to_rfc3339();

	// --- Assemble Final Payload (IExtensionHostInitData) ---
	let final_init_data_dto = json!({

		// Placeholder commit hash
		"commit": app_handle.package_info().version.to_string() + "-mountain-dev-commit-placeholder",

		"version": app_handle.package_info().version.to_string(),

		// or "stable", "insiders"
		"quality": "development",

		"parentPid": std::process::id(),

		"environment": {

			// TODO: Set based on actual debug mode for extensions
			"isExtensionDevelopmentDebug": false,

			"appName": app_handle.package_info().name.clone(),

			// as opposed to "web"
			"appHost": "desktop",

			"appRoot": path_to_uri_components_dto(app_root_heuristic_path, true),

			// TODO: Get from system/user settings if internationalization is added
			"appLanguage": "en",

			// Default to only logging telemetry events, not sending
			"isExtensionTelemetryLoggingOnly": true,

			// App URI scheme, e.g., "vscode", "landcode"
			"appUriScheme": app_handle.config().tauri.bundle.identifier.split('.').last().unwrap_or("landcode").to_string(),

			// For running specific extensions in dev mode (URIs)
			"extensionDevelopmentLocationURI": [],

			// For running extension tests (URI)
			"extensionTestsLocationURI": Value::Null,

			"globalStorageHome": path_to_uri_components_dto(global_storage_home_path, true),

			"workspaceStorageHome": path_to_uri_components_dto(workspace_storage_home_path, true),

			// Typically false for desktop applications
			"useHostProxy": false,

			// As per Cocoon's current main.ts, avoids file locking issues in some envs
			"skipWorkspaceStorageLock": true,

			// Map: extId -> proposalName[]
			"extensionEnabledProposedApi": enabled_proposed_apis_val,

			// Optional: e.g., [["publisher.someExtId", "Trace"]] to set per-extension log levels
			// "extensionLogLevel": []
		},

		// IStaticWorkspaceData or null
		"workspace": workspace_data_val,

		// No remote support in MVP
		"remote": { "isRemote": false, "authority": Value::Null, "connectionData": Value::Null },

		// Map log::LevelFilter to VS Code's LogLevel enum (number: Trace=0, Debug=1, Info=2, Warn=3, Error=4, Off=5, Critical=6)
		"logLevel": match log::max_level() {

			LevelFilter::Trace => 0,

			LevelFilter::Debug => 1,

			LevelFilter::Info => 2,

			LevelFilter::Warn  => 3,

			// Error and Critical map to VS Code's Error
			LevelFilter::Error => 4,

			LevelFilter::Off   => 5,

		},

		// URI
		"logsLocation": path_to_uri_components_dto(logs_loc_path, true),

		// For custom loggers provided by extensions (paths to log files)
		"loggers": [],

		// Start extension activation immediately after init
		"autoStart": true,

		// IExtensionDescriptionSnapshot
		"extensions": extensions_snapshot_data_dto,

		// For localized strings, not used in MVP
		"nlsBaseUrl": Value::Null,

		// IConfigurationInitData
		"configurationData": configuration_data_dto,

		"telemetryInfo": {

			"sessionId": session_id_str,

			// Unique machine identifier
			"machineId": machine_id_str.clone(),

			// Legacy telemetry ID, often same as machineId
			"sqmId": machine_id_str.clone(),

			// Another unique device ID
			"devDeviceId": machine_id_str,

			"firstSessionDate": first_session_date_str,

			// Example flag
			// "msftInternal": false
		},

		// OS type: Unknown = 0, Windows = 1, Mac = 2, Linux = 3
		"os": if cfg!(target_os = "windows") { 1 }


			else if cfg!(target_os = "macos") { 2 }


			else if cfg!(target_os = "linux") { 3 }


			else { 0 },

		// e.g., "x86_64", "aarch64"
		"arch": std::env::consts::ARCH.to_string(),

		// From VS Code's product.json concept, provides app-specific branding and info
		"product": {

			// Short name for UI elements
			"nameShort": "Land",

			// Full application name
			"nameLong": "Land Code Editor",

			// Often 'code' or similar for VS Code
			"applicationName": app_handle.package_info().name.clone(),

			"version": app_handle.package_info().version.to_string(),

			"commit": app_handle.package_info().version.to_string() + "-mountain-dev-commit-placeholder",

			// e.g., ".landcode"
			"dataFolderName": format!(".{}", app_handle.package_info().name.to_lowercase()),

			// No marketplace/gallery for MVP
			"extensionsGallery": Value::Null,

			// TODO: Add other product.json fields if needed by extensions (e.g., quality, date)
		},

		// UIKind.Desktop (vs UIKind.Web = 2)
		"uiKind": 1,

	});

	info!("[ProcMgmt InitData] Construction of IExtensionHostInitData complete.");

	final_init_data_dto
}
