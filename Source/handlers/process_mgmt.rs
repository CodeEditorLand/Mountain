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
//   - Constructing `IExtensionHostInitData` using `AppState`.
//   - Sending `initData` to Cocoon via `vine::send_notification_to_sidecar`.
// - Monitoring process exit and triggering cleanup.
//
// Key Interactions:
// - Called from `main.rs` Tauri setup hook.
// - Uses Tauri `PathResolver`.
// - Uses `tokio::process::Command`.
// - Interacts heavily with `Vine`.
// - Listens for Tauri events.
// - Accesses `AppState` to construct `initData`.
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

// For mapping log level in initData
use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	sync::mpsc as tokio_mpsc,
	time::timeout,
};
use url::Url;

use crate::{
	app_state::{AppState, ExtensionDescriptionState, WorkspaceFolderState},

	vine,
	// Not strictly needed for current functions, but available
	// handlers::error_utils,
};

// Not directly returned by public
// use Land_Common::errors::CommonError;

// functions here

// --- Main Launch Function ---

pub async fn launch_and_manage_cocoon<R:Runtime>(app_handle:AppHandle<R>) {
	info!("[ProcMgmt] Attempting to launch Cocoon Sidecar (Node.js Extension Host)...");

	let sidecar_id = "cocoon-main".to_string();

	// --- Determine Paths ---
	let path_resolver = app_handle.path_resolver();

	let node_path_opt = path_resolver.resolve_resource("bin/node");

	let script_path_opt = path_resolver.resolve_resource("scripts/cocoon/index.js");

	let (node_path_str, script_path_str) = match (node_path_opt, script_path_opt) {
		(Some(node_path), Some(script_path)) => {
			if !node_path.exists() || !node_path.is_file() {
				error!(
					"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Bundled Node.js executable not found or is not a file at \
					 resolved path: {}. Cocoon cannot start. Check Tauri 'resources' configuration and ensure the \
					 file is correctly bundled and executable.",
					node_path.display()
				);

				return;
			}

			if !script_path.exists() || !script_path.is_file() {
				error!(
					"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Cocoon main script not found or is not a file at resolved \
					 path: {}. Cocoon cannot start. Check Tauri 'resources' configuration.",
					script_path.display()
				);

				return;
			}

			info!("[ProcMgmt] Using bundled Node: {}", node_path.display());

			info!("[ProcMgmt] Using Cocoon script: {}", script_path.display());

			(
				node_path.to_string_lossy().into_owned(),
				script_path.to_string_lossy().into_owned(),
			)
		},

		(node_res, script_res) => {
			error!(
				"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Failed to resolve one or more bundled paths for Cocoon. Node \
				 resolved: {}, Script resolved: {}. Ensure 'bin/node' and 'scripts/cocoon/index.js' are correctly \
				 listed in tauri.conf.json's resources/externalBin and are present in the target directory.",
				node_res.is_some(),
				script_res.is_some()
			);

			return;
		},
	};

	// --- Prepare Command ---
	let mut command = Command::new(&node_path_str);

	command.arg(&script_path_str);

	// Pass Mountain's PID
	command.arg(format!("--parent-pid={}", std::process::id()));

	command.stdin(Stdio::piped());

	command.stdout(Stdio::piped());

	// Capture stderr
	command.stderr(Stdio::piped());

	// Ensure child process is killed when Command handle is dropped
	command.kill_on_drop(true);

	info!(
		"[ProcMgmt] Spawning Cocoon command: {} {} --parent-pid={}",
		node_path_str,
		script_path_str,
		std::process::id()
	);

	// --- Spawn Process ---
	match command.spawn() {
		Ok(mut child_process) => {
			let child_pid_opt = child_process.id();

			let child_pid_log = child_pid_opt.map_or_else(|| "unknown (spawned)".to_string(), |pid| pid.to_string());

			info!("[ProcMgmt] Cocoon process spawned successfully [OS PID: {}]", child_pid_log);

			if let Some(stderr_stream) = child_process.stderr.take() {
				let stderr_sidecar_id = sidecar_id.clone();

				let stderr_pid_for_log = child_pid_log.clone();

				tokio::spawn(async move {
					info!(
						"[Cocoon stderr ({})][PID: {}] Stderr monitoring task started.",
						stderr_sidecar_id, stderr_pid_for_log
					);

					let reader = BufReader::new(stderr_stream);

					let mut lines = reader.lines();

					while let Ok(Some(line)) = lines.next_line().await {
						error!("[Cocoon stderr ({})][PID: {}] {}", stderr_sidecar_id, stderr_pid_for_log, line);
					}

					info!(
						"[Cocoon stderr ({})][PID: {}] Stderr stream closed.",
						stderr_sidecar_id, stderr_pid_for_log
					);
				});
			} else {
				warn!("[ProcMgmt] Could not capture stderr for Cocoon sidecar '{}'.", sidecar_id);
			}

			match vine::setup_sidecar_communication(sidecar_id.clone(), child_process, app_handle.clone()) {
				Ok(_) => {
					info!(
						"[ProcMgmt] Vine communication setup initiated for '{}' [OS PID: {}]",
						sidecar_id, child_pid_log
					);

					spawn_init_data_sender(app_handle.clone(), sidecar_id.clone());
				},

				Err(vine_error) => {
					error!(
						"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Failed to setup Vine IPC for '{}' [OS PID: {}]: {}. \
						 Cocoon sidecar will be non-functional.",
						sidecar_id, child_pid_log, vine_error
					);
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

/// Spawns task that waits for ready signal and sends init data.
fn spawn_init_data_sender<R:Runtime>(app_handle:AppHandle<R>, sidecar_id:String) {
	tokio::spawn(async move {
		info!(
			"[ProcMgmt InitSender] Task for '{}' waiting for 'vine://sidecar/ready' signal...",
			sidecar_id
		);

		let (tx_ready_signal, mut rx_ready_signal) = tokio_mpsc::channel::<()>(1);

		let received_signal_flag = Arc::new(AtomicBool::new(false));

		// Clone for the listener closure
		let listener_app_handle = app_handle.clone();

		let listener_sidecar_id = sidecar_id.clone();

		let listener_received_flag = received_signal_flag.clone();

		let tauri_event_listener_id = app_handle.listen_global("vine://sidecar/ready", move |event| {
			if let Some(payload_str) = event.payload() {
				match serde_json::from_str::<String>(payload_str) {
					Ok(ready_id_from_event) if ready_id_from_event == listener_sidecar_id => {
						if listener_received_flag
							.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
							.is_ok()
						{
							info!(
								"[ProcMgmt InitSender] Received 'vine://sidecar/ready' signal for '{}'.",
								listener_sidecar_id
							);

							if tx_ready_signal.try_send(()).is_err() {
								// Check result of try_send
								error!(
									"[ProcMgmt InitSender] Failed to send internal ready confirmation for '{}' \
									 (receiver likely dropped due to timeout).",
									listener_sidecar_id
								);
							}
						} else {
							warn!(
								"[ProcMgmt InitSender] Duplicate 'vine://sidecar/ready' signal for '{}'. Ignoring.",
								listener_sidecar_id
							);
						}
					},

					Ok(other_id) => {
						// Signal for a different sidecar
						trace!(
							"[ProcMgmt InitSender] 'ready' signal for other sidecar '{}', expecting '{}'.",
							other_id, listener_sidecar_id
						);
					},

					Err(e) => {
						// Payload parsing error
						error!(
							"[ProcMgmt InitSender] Failed to parse 'vine://sidecar/ready' payload as string: '{}'. \
							 Payload: {}",
							e, payload_str
						);
					},
				}
			} else {
				// Event with no payload
				warn!(
					"[ProcMgmt InitSender] Received 'vine://sidecar/ready' event with no payload for '{}'.",
					listener_sidecar_id
				);
			}
		});

		debug!(
			"[ProcMgmt InitSender] Listening for 'vine://sidecar/ready' from '{}' (Tauri listener ID: {}).",
			sidecar_id, tauri_event_listener_id
		);

		let ready_timeout_duration = Duration::from_secs(30);

		match timeout(ready_timeout_duration, rx_ready_signal.recv()).await {
			Ok(Some(_)) => {
				info!(
					"[ProcMgmt InitSender] Internal ready confirmation received for '{}'. Proceeding to send initData.",
					sidecar_id
				);
			},

			Ok(None) => {
				// MPSC channel closed before message, tx_ready_signal likely dropped
				error!(
					"[ProcMgmt InitSender] Internal ready signal channel for '{}' closed unexpectedly. Aborting \
					 initData send.",
					sidecar_id
				);

				app_handle.unlisten(tauri_event_listener_id);

				// Ensure cleanup
				vine::unregister_sidecar(&sidecar_id);

				return;
			},

			Err(_) => {
				// Timeout
				error!(
					"[ProcMgmt InitSender] Timed out ({:?}) waiting for ready signal from '{}'. Sidecar might have \
					 failed. Aborting initData send.",
					ready_timeout_duration, sidecar_id
				);

				app_handle.unlisten(tauri_event_listener_id);

				// Important to unregister if sidecar is unresponsive
				vine::unregister_sidecar(&sidecar_id);

				return;
			},
		}

		// Unlisten once signal processed or timed out
		app_handle.unlisten(tauri_event_listener_id);

		info!(
			"[ProcMgmt InitSender] Constructing and sending initExtensionHost data to '{}'...",
			sidecar_id
		);

		let init_data_payload = construct_init_data(&app_handle);

		// Note: construct_init_data uses .expect() on locks, which will panic if
		// poisoned. For production, this could be changed to return Result and
		// handled here.
		trace!(
			"[ProcMgmt InitSender] Constructed initData for '{}': (Top-level keys: {:?})",
			sidecar_id,
			init_data_payload.as_object().map(|o| o.keys().collect::<Vec<_>>())
		);

		match vine::send_notification_to_sidecar(&sidecar_id, "initExtensionHost".to_string(), init_data_payload).await
		{
			Ok(_) => {
				info!(
					"[ProcMgmt InitSender] initExtensionHost notification sent successfully to '{}'.",
					sidecar_id
				)
			},

			Err(e) => {
				error!(
					"[ProcMgmt InitSender] Failed to send initExtensionHost notification to '{}': {}. Sidecar may not \
					 initialize correctly.",
					sidecar_id, e
				)
			},
		}
	});
}

/// Constructs the `IExtensionHostInitData` payload.
/// This function needs access to `AppState` and `PathResolver`.
/// Panics on lock failure are considered acceptable during this critical
/// startup phase, but could be refactored to return `Result` for more graceful
/// degradation if needed.
fn construct_init_data<R:Runtime>(app_handle:&AppHandle<R>) -> Value {
	let app_state = app_handle.state::<AppState>();

	info!("[ProcMgmt InitData] Constructing IExtensionHostInitData...");

	// Helper to create UriComponents JSON Value from a PathBuf
	let path_to_uri_comp_val_fn = |p:PathBuf, is_dir:bool| -> Value {
		let url_res = if is_dir { Url::from_directory_path(&p) } else { Url::from_file_path(&p) };

		let url = url_res.unwrap_or_else(|_| {
			warn!(
				"[ProcMgmt InitData] Failed to create URL from path: {}. Using lossy string.",
				p.display()
			);

			Url::parse(&format!("file:///{}", p.to_string_lossy().replace('\\', "/")))
				.expect("Fallback URL parse for initData path failed")
		});

		json!({

			"scheme": url.scheme(),

			// Ensure authority is string even if empty
			"authority": url.host_str().unwrap_or(""),

			"path": url.path(),

			// Keep query if present
			"query": url.query(),

			// Keep fragment if present
			"fragment": url.fragment(),

			"external": url.to_string(),

			"$mid": 1
		})
	};

	// --- Workspace Data ---
	let (
		workspace_id,
		workspace_name,
		workspace_config_uri_components,
		workspace_folders_data,
		is_transient,
		is_untitled,
	) = {
		let folders_guard = app_state.workspace_folders.lock().expect("Lock workspace_folders for initData");

		let config_path_guard = app_state
			.workspace_config_path
			.lock()
			.expect("Lock workspace_config_path for initData");

		let id = app_state.get_workspace_id_string().unwrap_or_else(|e| {
			warn!("[ProcMgmt InitData] Failed to get workspace ID string: {}. Using default.", e);

			"FALLBACK_WORKSPACE_ID".to_string()
		});

		let name = app_state.get_workspace_name().unwrap_or_else(|e| {
			warn!("[ProcMgmt InitData] Failed to get workspace name: {}. Using default.", e);

			"Untitled Workspace".to_string()
		});

		// Mountain workspaces are not typically transient like in remote dev
		let transient = false;

		let untitled = config_path_guard.is_none()
			&& folders_guard.len() <= 1
			&& (folders_guard.first().map_or(true, |f| f.uri.scheme() == "untitled"));

		let config_components = config_path_guard.as_ref().map(|p| path_to_uri_comp_val_fn(p.clone(), false));

		let folders_components:Vec<Value> = folders_guard
			.iter()
			.map(|f_state:&WorkspaceFolderState| {
				let f_uri_comp = json!({

					"scheme": f_state.uri.scheme(),

					"authority": f_state.uri.host_str().unwrap_or(""),

					"path": f_state.uri.path(),

					"query": f_state.uri.query(),

					"fragment": f_state.uri.fragment(),

					"external": f_state.uri.to_string(),

					"$mid": 1
				});

				json!({ "uri": f_uri_comp, "name": f_state.name, "index": f_state.index })
			})
			.collect();

		debug!(
			"[ProcMgmt InitData] Workspace: ID='{}', Name='{}', Config Path Present: {}, Num Folders={}, Untitled: {}",
			id,
			name,
			config_components.is_some(),
			folders_components.len(),
			untitled
		);

		(id, name, config_components, folders_components, transient, untitled)
	};

	let workspace_data_val =
		if workspace_folders_data.is_empty() && workspace_config_uri_components.is_none() && !is_untitled {
			// No workspace if no folders, no config, and not explicitly untitled
			Value::Null
		} else {
			json!({

				"id": workspace_id,

				"name": workspace_name,

				"configuration": workspace_config_uri_components.unwrap_or(Value::Null),

				"folders": workspace_folders_data,

				"transient": is_transient,

				"isUntitled": is_untitled
			})
		};

	// --- Extensions Data ---
	let (all_extensions_dto, my_extensions_ids_dto, activation_events_dto) = {
		let scanned_extensions_guard = app_state
			.scanned_extensions
			.lock()
			.expect("Lock scanned_extensions for initData");

		let mut all_ext_descs:Vec<Value> = Vec::new();

		// IDs of extensions for *this* host
		let mut my_ext_ids:Vec<Value> = Vec::new();

		let mut act_events:HashMap<String, Vec<String>> = HashMap::new();

		for (ext_full_id_str, ext_desc_state) in scanned_extensions_guard.iter() {
			// Assuming all scanned extensions run in this primary host for now
			match serde_json::to_value(ext_desc_state.clone()) {
				Ok(serialized_desc) => {
					all_ext_descs.push(serialized_desc);

					// identifier is {value, uuid}

					my_ext_ids.push(ext_desc_state.identifier.clone());

					if let Some(events) = &ext_desc_state.activation_events {
						if !events.is_empty() {
							act_events.insert(ext_full_id_str.clone(), events.clone());
						}
					}
				},

				Err(e) => {
					error!(
						"[ProcMgmt InitData] Failed to serialize ExtensionDescriptionState for {}: {}",
						ext_full_id_str, e
					)
				},
			}
		}

		info!(
			"[ProcMgmt InitData] Processed {} scanned extensions for initData.",
			all_ext_descs.len()
		);

		(all_ext_descs, my_ext_ids, act_events)
	};

	let extensions_snapshot_data = json!({

		// Simple versioning for now
		"versionId": 1,

		"allExtensions": all_extensions_dto,

		"myExtensions": my_extensions_ids_dto,

		"activationEvents": activation_events_dto
	});

	// --- Configuration Data ---
	let configuration_data_dto = {
		let config_guard = app_state.configuration.lock().expect("Lock configuration for initData");

		json!({

			"effective": config_guard.data.clone(),

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
		})
	};

	trace!(
		"[ProcMgmt InitData] Configuration data DTO effective keys: {}",
		configuration_data_dto["effective"].as_object().map_or(0, |o| o.keys().len())
	);

	// --- Paths ---
	let path_resolver = app_handle.path_resolver();

	let logs_loc = path_resolver
		.app_log_dir()
		// Fallback for logs
		.unwrap_or_else(|| PathBuf::from("./dev_logs_fallback"));

	let app_data_dir_base = path_resolver
		.app_data_dir()
		.unwrap_or_else(|| PathBuf::from("./dev_appdata_fallback"));

	let user_data_root = app_data_dir_base.join("User");

	let global_storage_home = user_data_root.join("globalStorage");

	let ws_storage_home_id = app_state
		.get_workspace_id_string()
		.unwrap_or_else(|_| "NO_WORKSPACE_ID_FOR_STORAGE".to_string());

	let workspace_storage_home = user_data_root.join("workspaceStorage").join(ws_storage_home_id);

	let app_root_fallback = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

	let app_root_path = path_resolver
		.app_config_dir()
		.or_else(|| path_resolver.app_data_dir())
		.unwrap_or(app_root_fallback);

	// --- Proposed APIs ---
	let enabled_proposed_apis_val = {
		let guard = app_state.enabled_proposed_apis.lock().expect("Lock proposed APIs for initData");

		if guard.len() == 1 && guard.contains_key("*") {
			json!(guard.get("*").unwrap())
		} else {
			json!(*guard)
		}
	};

	debug!(
		"[ProcMgmt InitData] Enabled Proposed APIs for initData: {:?}",
		enabled_proposed_apis_val
	);

	// --- Telemetry Info ---
	// Using Tauri's instance ID
	let machine_id = app_handle.manager().instance_id().to_string();

	let session_id = uuid::Uuid::new_v4().to_string();

	// ISO 8601 format
	let first_session_date = chrono::Utc::now().to_rfc3339();

	// --- Assemble Final Payload (IExtensionHostInitData) ---
	let final_init_data = json!({

		"commit": app_handle.package_info().version.to_string() + "-mountain-dev",

		"version": app_handle.package_info().version.to_string(),

		// or "stable", "insiders"
		"quality": "development",

		"parentPid": std::process::id(),

		"environment": {

			// TODO: Set based on actual debug mode
			"isExtensionDevelopmentDebug": false,

			"appName": app_handle.package_info().name.clone(),

			// as opposed to "web"
			"appHost": "desktop",

			"appRoot": path_to_uri_comp_val_fn(app_root_path, true),

			// TODO: Get from system/user settings
			"appLanguage": "en",

			// Default to only logging telemetry events, not sending
			"isExtensionTelemetryLoggingOnly": true,

			"appUriScheme": app_handle.config().tauri.bundle.identifier.split('.').last().unwrap_or("landcode").to_string(),

			// For running specific extensions in dev mode
			"extensionDevelopmentLocationURI": [],

			// For running extension tests
			"extensionTestsLocationURI": Value::Null,

			"globalStorageHome": path_to_uri_comp_val_fn(global_storage_home, true),

			"workspaceStorageHome": path_to_uri_comp_val_fn(workspace_storage_home, true),

			// Typically false for desktop
			"useHostProxy": false,

			// As per Cocoon's current main.ts
			"skipWorkspaceStorageLock": true,

			"extensionEnabledProposedApi": enabled_proposed_apis_val,

			// Optional: e.g., [["publisher.someExtId", "Trace"]]
			// "extensionLogLevel": []
		},

		// IStaticWorkspaceData or null
		"workspace": workspace_data_val,

		"remote": { "isRemote": false, "authority": Value::Null, "connectionData": Value::Null },

		// Map log::LevelFilter to VS Code's LogLevel enum (number)
		"logLevel": match log::max_level() {

			LevelFilter::Trace => 0, LevelFilter::Debug => 1, LevelFilter::Info => 2,

			// VS Code LogLevel.Off is 5, Critical is 6
			LevelFilter::Warn  => 3, LevelFilter::Error => 4, LevelFilter::Off   => 5,

		},

		"logsLocation": path_to_uri_comp_val_fn(logs_loc, true),

		// For custom loggers provided by extensions
		"loggers": [],

		// To start extension activation immediately
		"autoStart": true,

		// IExtensionDescriptionSnapshot
		"extensions": extensions_snapshot_data,

		// For localized strings, not used in MVP
		"nlsBaseUrl": Value::Null,

		// IConfigurationInitData
		"configurationData": configuration_data_dto,

		"telemetryInfo": {

			"sessionId": session_id,

			// Use the same unique ID for these
			"machineId": machine_id.clone(),

			// sqmId is legacy
			"sqmId": machine_id.clone(),

			// devDeviceId is another unique ID
			"devDeviceId": machine_id,

			"firstSessionDate": first_session_date,

			// Example
			// "msftInternal": false
		},

		// Linux
		"os": if cfg!(target_os = "linux") { 3 }

			// MacOS
			else if cfg!(target_os = "macos") { 2 }

			// Windows
			else if cfg!(target_os = "windows") { 1 }

			// UnknownOS
			else { 0 },

		// e.g., "x86_64", "aarch64"
		"arch": std::env::consts::ARCH.to_string(),

		// From VS Code's product.json concept
		"product": {

			// Short name for UI elements
			"nameShort": "Land",

			// Full application name
			"nameLong": "Land Code Editor",

			// Usually 'code' or similar for VS Code
			"applicationName": app_handle.package_info().name.clone(),

			"version": app_handle.package_info().version.to_string(),

			// Placeholder commit
			"commit": app_handle.package_info().version.to_string() + "-mountain-dev-commit",

			// e.g., ".landcode"
			"dataFolderName": format!(".{}", app_handle.package_info().name.to_lowercase()),

			// No gallery for MVP
			"extensionsGallery": Value::Null,

		},

		// UIKind.Desktop (vs UIKind.Web = 2)
		"uiKind": 1,

	});

	info!("[ProcMgmt InitData] Construction complete.");

	final_init_data
}
