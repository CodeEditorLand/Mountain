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
//   - Constructing `IExtensionHostInitData` using `AppState` (now including
//     scanned extensions and proposed API configs).
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
	// For activation_events_dto in construct_init_data
	collections::HashMap,

	path::PathBuf,

	process::Stdio,

	sync::Arc,

	sync::atomic::{AtomicBool, Ordering},

	time::Duration,
};

use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	sync::mpsc as tokio_mpsc,
	time::timeout,
};
// For constructing file URLs correctly
use url::Url;

use crate::{
	app_state::{AppState, ExtensionDescriptionState, WorkspaceFolderState},
	vine,
};

// --- Main Launch Function ---

pub async fn launch_and_manage_cocoon<R:Runtime>(app_handle:AppHandle<R>) {
	info!("[ProcMgmt] Launching Cocoon Sidecar...");

	let sidecar_id = "cocoon-main".to_string();

	// --- Determine Paths ---
	let node_path_res = app_handle.path_resolver().resolve_resource("bin/node");

	let script_path_res = app_handle.path_resolver().resolve_resource("scripts/cocoon/index.js");

	let (node_path_str, script_path_str) = match (node_path_res, script_path_res) {
		(Some(node), Some(script)) => {
			info!("[ProcMgmt] Using bundled Node: {}", node.display());

			info!("[ProcMgmt] Using Cocoon script: {}", script.display());

			(node.to_string_lossy().into_owned(), script.to_string_lossy().into_owned())
		},

		(node_res, script_res) => {
			error!(
				"[ProcMgmt] CRITICAL: Failed to resolve bundled paths. Node found: {}, Script found: {}. Check \
				 tauri.conf.json resources/externalBin and ensure paths are correct.",
				node_res.is_some(),
				script_res.is_some()
			);

			return;
		},
	};

	// --- Prepare Command ---
	let mut command = Command::new(&node_path_str);

	command.arg(&script_path_str);

	command.stdin(Stdio::piped());

	command.stdout(Stdio::piped());

	command.stderr(Stdio::piped());

	command.kill_on_drop(true);

	info!("[ProcMgmt] Spawning command: {} {}", node_path_str, script_path_str);

	// --- Spawn Process ---
	match command.spawn() {
		Ok(mut child) => {
			let child_pid_log = child.id().map_or_else(|| "unknown (spawned)".into(), |id| id.to_string());

			info!("[ProcMgmt] Cocoon process spawned successfully [PID: {}]", child_pid_log);

			if let Some(stderr) = child.stderr.take() {
				let stderr_sidecar_id = sidecar_id.clone();

				let stderr_pid_log = child_pid_log.clone();

				tokio::spawn(async move {
					let reader = BufReader::new(stderr);

					let mut lines = reader.lines();

					while let Ok(Some(line)) = lines.next_line().await {
						error!("[Cocoon stderr ({})][PID: {}] {}", stderr_sidecar_id, stderr_pid_log, line);
					}

					info!(
						"[Cocoon stderr ({})][PID: {}] Stderr stream closed.",
						stderr_sidecar_id, stderr_pid_log
					);
				});
			} else {
				warn!("[ProcMgmt] Could not capture stderr for Cocoon sidecar '{}'.", sidecar_id);
			}

			match vine::setup_sidecar_communication(sidecar_id.clone(), child, app_handle.clone()) {
				Ok(_) => {
					info!(
						"[ProcMgmt] Vine communication setup initiated for '{}' [PID: {}]",
						sidecar_id, child_pid_log
					);

					spawn_init_data_sender(app_handle.clone(), sidecar_id.clone());
				},

				Err(e) => {
					error!(
						"[ProcMgmt] Failed to setup Vine communication for '{}' [PID: {}]: {}",
						sidecar_id, child_pid_log, e
					);
				},
			}
		},

		Err(e) => {
			error!(
				"[ProcMgmt] CRITICAL: Failed to spawn Cocoon process using Node path '{}': {}",
				node_path_str, e
			);
		},
	}
}

/// Spawns task that waits for ready signal and sends init data.
fn spawn_init_data_sender<R:Runtime>(app_handle:AppHandle<R>, sidecar_id:String) {
	tokio::spawn(async move {
		info!(
			"[ProcMgmt InitSender] Task started for '{}'. Waiting for 'vine://sidecar/ready' signal...",
			sidecar_id
		);

		let (tx_ready_signal, mut rx_ready_signal) = tokio_mpsc::channel::<()>(1);

		let received_signal_flag = Arc::new(AtomicBool::new(false));

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

							if let Err(e) = tx_ready_signal.try_send(()) {
								error!(
									"[ProcMgmt InitSender] Failed to send internal ready confirmation for '{}': {}",
									listener_sidecar_id, e
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
						trace!(
							"[ProcMgmt InitSender] Received 'vine://sidecar/ready' for different sidecar '{}', \
							 expecting '{}'.",
							other_id, listener_sidecar_id
						);
					},

					Err(e) => {
						error!(
							"[ProcMgmt InitSender] Failed to parse 'vine://sidecar/ready' payload '{}': {}",
							payload_str, e
						);
					},
				}
			} else {
				warn!(
					"[ProcMgmt InitSender] Received 'vine://sidecar/ready' event with no payload for '{}'.",
					listener_sidecar_id
				);
			}
		});

		debug!(
			"[ProcMgmt InitSender] Listening for 'vine://sidecar/ready' from '{}' with listener ID: {}",
			sidecar_id, tauri_event_listener_id
		);

		let ready_timeout = Duration::from_secs(30);

		match timeout(ready_timeout, rx_ready_signal.recv()).await {
			Ok(Some(_)) => {
				info!(
					"[ProcMgmt InitSender] Successfully received internal ready confirmation for '{}'.",
					sidecar_id
				);
			},

			Ok(None) => {
				error!(
					"[ProcMgmt InitSender] Internal ready signal channel closed unexpectedly for '{}'.",
					sidecar_id
				);

				app_handle.unlisten(tauri_event_listener_id);

				vine::unregister_sidecar(&sidecar_id);

				return;
			},

			Err(_) => {
				// Timeout
				error!(
					"[ProcMgmt InitSender] Timed out ({:?}) waiting for ready signal from '{}'.",
					ready_timeout, sidecar_id
				);

				app_handle.unlisten(tauri_event_listener_id);

				vine::unregister_sidecar(&sidecar_id);

				return;
			},
		}

		app_handle.unlisten(tauri_event_listener_id);

		info!(
			"[ProcMgmt InitSender] Sidecar '{}' is ready. Constructing and sending init data...",
			sidecar_id
		);

		let init_data = construct_init_data(&app_handle);

		trace!(
			"[ProcMgmt InitSender] Constructed initData for '{}': (Top-level keys: {:?})",
			sidecar_id,
			init_data.as_object().map(|o| o.keys().collect::<Vec<_>>())
		);

		match vine::send_notification_to_sidecar(&sidecar_id, "initExtensionHost".to_string(), init_data).await {
			Ok(_) => info!("[ProcMgmt InitSender] Init data sent successfully to '{}'.", sidecar_id),

			Err(e) => error!("[ProcMgmt InitSender] Failed to send init data to '{}': {}", sidecar_id, e),
		}
	});
}

/// Constructs the `IExtensionHostInitData` payload.
fn construct_init_data<R:Runtime>(app_handle:&AppHandle<R>) -> Value {
	let app_state = app_handle.state::<AppState>();

	info!("[ProcMgmt InitData] Constructing IExtensionHostInitData...");

	// --- Workspace Data ---
	let (
		workspace_id,
		workspace_name,
		workspace_config_uri_components,
		workspace_folders_data,
		is_transient,
		is_untitled,
	) = {
		let folders_guard = app_state.workspace_folders.lock().expect("Lock workspace_folders");

		let config_path_guard = app_state.workspace_config_path.lock().expect("Lock workspace_config_path");

		let id = app_state.get_workspace_id_string().unwrap_or_else(|e| {
			warn!("[ProcMgmt InitData] Failed to get workspace ID string: {}. Using default.", e);

			"FALLBACK_WORKSPACE_ID".to_string()
		});

		let name = app_state.get_workspace_name().unwrap_or_else(|e| {
			warn!("[ProcMgmt InitData] Failed to get workspace name: {}. Using default.", e);

			"Untitled Workspace".to_string()
		});

		let transient = false;

		let untitled = config_path_guard.is_none() && folders_guard.len() <= 1;

		let path_to_uri_comp_val_fn = |p:&PathBuf, is_dir:bool| -> Value {
			let url_res = if is_dir { Url::from_directory_path(p) } else { Url::from_file_path(p) };

			let url = url_res.unwrap_or_else(|_| {
				Url::parse(&format!("file:///{}", p.to_string_lossy().replace('\\', "/"))).expect("Fallback URL parse")
			});

			json!({ "scheme": url.scheme(), "path": url.path(), "external": url.to_string(), "$mid": 1 })
		};

		let config_components = config_path_guard.as_ref().map(|p| path_to_uri_comp_val_fn(p, false));

		let folders_components:Vec<Value> = folders_guard
			.iter()
			.map(|f| {
				json!({


					"uri": { "scheme": f.uri.scheme(), "path": f.uri.path(), "external": f.uri.to_string(), "$mid": 1 },

					"name": f.name, "index": f.index
				})
			})
			.collect();

		debug!(
			"[ProcMgmt InitData] Workspace: ID='{}', Name='{}', Config Path Present: {}, Num Folders={}",
			id,
			name,
			config_components.is_some(),
			folders_components.len()
		);

		(id, name, config_components, folders_components, transient, untitled)
	};

	let workspace_data = if workspace_folders_data.is_empty() && workspace_config_uri_components.is_none() {
		Value::Null
	} else {
		json!({ "id": workspace_id, "name": workspace_name, "configuration": workspace_config_uri_components.unwrap_or(Value::Null), "folders": workspace_folders_data, "transient": is_transient, "isUntitled": is_untitled })
	};

	// --- Extensions Data ---
	let (all_extensions_dto, my_extensions_ids_dto, activation_events_dto) = {
		let scanned_extensions_guard = app_state.scanned_extensions.lock().expect("Lock scanned_extensions");

		let mut all_ext_descs:Vec<Value> = Vec::new();

		let mut my_ext_ids:Vec<Value> = Vec::new();

		let mut act_events:HashMap<String, Vec<String>> = HashMap::new();

		for (ext_full_id_str, ext_desc_state) in scanned_extensions_guard.iter() {
			match serde_json::to_value(ext_desc_state.clone()) {
				Ok(serialized_desc) => {
					all_ext_descs.push(serialized_desc);

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

	// IExtensionHostInitData.extensions is IExtensionDescriptionSnapshot
	let extensions_snapshot_data = json!({


		// Placeholder, VS Code uses this for deltas for extension changes.
		"versionId": 1,

		"allExtensions": all_extensions_dto,

		// List of ExtensionIdentifier DTOs
		"myExtensions": my_extensions_ids_dto,

		// Map from extension ID to its activation events
		"activationEvents": activation_events_dto
	});

	// --- Settings Data (Effective configuration for IConfigurationInitData) ---
	// Note: IExtensionHostInitData itself doesn't directly take settings.
	// It takes `logLevel` and `logsLocation`.
	// The `settings` are part of IConfigurationInitData which is sent separately
	// by `$acceptConfigurationChanged` or similar later if needed, or ext host
	// requests it. However, some older protocols might have bundled it.
	// For current VS Code, `settings` is NOT part of `IExtensionHostInitData`.
	// It's managed by the ConfigurationService which syncs via
	// `$acceptConfigurationChanged`. The `AppState.configuration` is the
	// *effective* merged configuration. We'll include it under a
	// `configurationData` field to be passed to `ExtHostConfiguration` constructor.
	let configuration_data_dto = {
		let config_guard = app_state.configuration.lock().expect("Lock configuration for initData");

		json!({


			"effective": config_guard.data.clone(),

			// Stubs for now
			"defaults": { "contents": {} },

			"user": { "contents": {} },

			"workspace": { "contents": {} },

			"folders": [],

			"memory": { "contents": {} },

			"policy": Value::Null,

			"configurationScopes": []
		})
	};

	trace!(
		"[ProcMgmt InitData] Configuration data DTO effective keys: {}",
		configuration_data_dto["effective"].as_object().map_or(0, |o| o.keys().len())
	);

	// --- Paths ---
	let path_resolver = app_handle.path_resolver();

	let logs_loc = path_resolver.app_log_dir().unwrap_or_else(|| PathBuf::from("./dev_logs"));

	let app_data_d = path_resolver.app_data_dir().unwrap_or_else(|| PathBuf::from("./dev_appdata"));

	// User data root
	let user_data_d = app_data_d.join("User");

	let global_store_home = user_data_d.join("globalStorage");

	let ws_store_home_id_str = app_state
		.get_workspace_id_string()
		.unwrap_or_else(|_| "NO_WORKSPACE_ID_FOR_STORAGE".to_string());

	let ws_store_home = user_data_d.join("workspaceStorage").join(ws_store_home_id_str);

	let app_root_p = path_resolver
		.app_config_dir()
		.or_else(|| path_resolver.app_data_dir())
		.unwrap_or_else(|| {
			std::env::current_exe()
				.ok()
				.and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
				.unwrap_or_else(|| PathBuf::from("."))
		});

	let path_to_uri_comp_val_for_init_data_fn = |p:PathBuf, is_dir:bool| -> Value {
		let url_res = if is_dir { Url::from_directory_path(&p) } else { Url::from_file_path(&p) };

		let url = url_res.unwrap_or_else(|_| {
			Url::parse(&format!("file:///{}", p.to_string_lossy().replace('\\', "/")))
				.expect("Fallback URL parse for initData path")
		});

		json!({ "scheme": url.scheme(), "path": url.path(), "external": url.to_string(), "$mid": 1 })
	};

	// --- Proposed APIs ---
	let enabled_proposed_apis_val = {
		let guard = app_state.enabled_proposed_apis.lock().expect("Lock proposed APIs");

		// VS Code expects `string[]` for global '*' or `{[extId]: string[]}`
		// This structure should align with `IEnabledApiProposals` in VS Code.
		// If only '*' is present, send its array. Otherwise, send the map.
		if guard.len() == 1 && guard.contains_key("*") {
			json!(guard.get("*").unwrap())
		} else {
			json!(*guard)
			// This serializes the HashMap as a JSON object
		}
	};

	debug!(
		"[ProcMgmt InitData] Enabled Proposed APIs for initData: {:?}",
		enabled_proposed_apis_val
	);

	// --- Telemetry Info ---
	// Use get_window which returns Option
	let main_window_opt = app_handle.get_window("main");

	let machine_id = main_window_opt.as_ref().map_or_else(
		|| {
			warn!("[ProcMgmt InitData] Main window not found for machineId, generating UUID.");

			uuid::Uuid::new_v4().to_string()
		},
		// Tauri instance ID as a stand-in
		|w| w.instance_id().to_string(),
	);

	let session_id = uuid::Uuid::new_v4().to_string();

	let first_session_date = chrono::Utc::now().to_rfc3339();

	// --- Assemble Final Payload for IExtensionHostInitData ---
	let final_init_data = json!({


		// More specific commit
		"commit": app_handle.package_info().version.to_string() + "-cocoon-dev-commit-final",

		"version": app_handle.package_info().version.to_string(),

		"quality": "development",

		"parentPid": std::process::id(),

		"environment": {


			// TODO: Set based on debug/dev mode
			"isExtensionDevelopmentDebug": false,

			"appName": app_handle.package_info().name.clone(),

			"appHost": "desktop",

			"appRoot": path_to_uri_comp_val_for_init_data_fn(app_root_p, true),

			// TODO: Get from system or config
			"appLanguage": "en",

			// Default to logging only for telemetry
			"isExtensionTelemetryLoggingOnly": true,

			"appUriScheme": app_handle.config().tauri.bundle.identifier.split('.').last().unwrap_or("landcode").to_string(),

			// For running specific extensions in dev mode
			"extensionDevelopmentLocationURI": [],

			// For running extension tests
			"extensionTestsLocationURI": Value::Null,

			"globalStorageHome": path_to_uri_comp_val_for_init_data_fn(global_store_home, true),

			"workspaceStorageHome": path_to_uri_comp_val_for_init_data_fn(ws_store_home, true),

			"useHostProxy": false,

			// Based on Cocoon's index.ts
			"skipWorkspaceStorageLock": true,

			"extensionEnabledProposedApi": enabled_proposed_apis_val,

			// Example: [["publisher.someExtId", "Trace"]]
			// "extensionLogLevel": []
		},

		// IStaticWorkspaceData
		"workspace": workspace_data,

		"remote": { "isRemote": false, "authority": Value::Null, "connectionData": Value::Null },

		// Use .to_level() for Option<Level>
		"logLevel": match log::max_level().to_level() {


			Some(log::Level::Trace) => 0, Some(log::Level::Debug) => 1, Some(log::Level::Info) => 2,

			// Off
			Some(log::Level::Warn) => 3, Some(log::Level::Error) => 4, None => 6,

		},

		"logsLocation": path_to_uri_comp_val_for_init_data_fn(logs_loc, true),

		// For custom loggers provided by extensions
		"loggers": [],

		// To start extension activation
		"autoStart": true,

		// IExtensionDescriptionSnapshot
		"extensions": extensions_snapshot_data,

		"nlsBaseUrl": Value::Null,

		// Pass IConfigurationInitData here
		"configurationData": configuration_data_dto,

		"telemetryInfo": {


			// sqmId is legacy
			"sessionId": session_id, "machineId": machine_id.clone(), "sqmId": machine_id.clone(),

			// devDeviceId is another unique ID
			"devDeviceId": machine_id,

			"firstSessionDate": first_session_date,

			// Example
			// "msftInternal": false
		},

		"os": if cfg!(target_os = "linux") { 3 }

			else if cfg!(target_os = "macos") { 2 }

			else if cfg!(target_os = "windows") { 1 }

			// UnknownOS
			else { 0 },

		"arch": std::env::consts::ARCH,

		// From VS Code's product.json concept
		"product": {


			"nameShort": "Land", "nameLong": "Land Code Editor",

			"applicationName": app_handle.package_info().name.clone(),

			"version": app_handle.package_info().version.to_string(),

			"commit": app_handle.package_info().version.to_string() + "-cocoon-dev-commit-final",

			// e.g. ".landcodeeditor"
			"dataFolderName": format!(".{}", app_handle.package_info().name.to_lowercase()),

			// No gallery for MVP
			"extensionsGallery": Value::Null,

		},

		// UIKind.Desktop (vs UIKind.Web)
		"uiKind": 1,

	});

	info!("[ProcMgmt InitData] Construction complete. Sending to Cocoon sidecar.");

	final_init_data
}
