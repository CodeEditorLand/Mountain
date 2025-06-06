// ---------------------------------------------------------------------------------------------
// Mountain Process Management Handlers 
// --------------------------------------------------------------------------------------------
// Responsible for launching, managing the lifecycle of, and establishing
// initial communication with sidecar processes, specifically the Cocoon Node.js
// extension host.
//
// Responsibilities:
// - Locating executables/scripts (Node.js, Cocoon/index.js).
// - Spawning the Cocoon sidecar process.
// - Configuring stdio pipes for IPC.
// - Initiating `Vine` IPC setup.
// - Orchestrating the initial handshake and data transfer:
//   - Waiting for `vine://sidecar/ready` event from Cocoon.
//   - Sending main `initExtensionHost` data (request-response).
//   - Sending FileSystem provider capabilities ($acceptProviderInfos).
//   - Sending full initial configuration model ($initializeConfiguration).
// - Monitoring process exit and triggering cleanup.
//
// Key Interactions:
// - Called from `main.rs` Tauri setup hook.
// - Uses Tauri `PathResolver` and `AppHandle`.
// - Uses `tokio::process::Command` for spawning.
// - Interacts heavily with `Vine` for IPC.
// - Listens for Tauri events.
// - Accesses `AppState` to construct `IExtensionHostInitData` and other
//   payloads.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	path::PathBuf,
	process::Stdio,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering as AtomicOrdering},
	},
	time::Duration,
};

// DTOs and Enums from Land_Common
use Land_Common::{
	config_effects::{ConfigurationScope, IConfigurationInitDataDto},
	fs_effects::FileSystemProviderCapabilities, // This is likely a u32 bitmask
	ipc_effects::ProxyTarget,
};
use chrono::Utc; // For dates in initData
use log::{LevelFilter as LogLevelFilter, debug, error, info, trace, warn}; // Renamed to avoid conflict
use serde_json::{Value, json};
use tauri::{AppHandle, Listener, Manager, Runtime}; // Removed Wry as R: Runtime covers it
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	sync::mpsc as tokio_mpsc,
	time::timeout,
};
use url::Url;
use uuid::Uuid; // For session IDs

use crate::{
	app_state::{AppState, WorkspaceFolderState}, // Add ExtensionDescriptionState if needed by construct_
	environment::MountainEnvironment,            // For get_file_provider_capabilities
	handlers::sky_dtos,                          // For ProductConfigurationDto within initData
	runtime::AppRuntime,                         // To get MountainEnvironment
	vine,
};

// --- Main Launch Function ---
pub async fn launch_and_manage_cocoon_sidecar<R:Runtime>(app_handle:AppHandle<R>) {
	info!("[ProcMgmt] Attempting to launch Cocoon Sidecar...");
	let sidecar_id = "cocoon-main".to_string();
	let path_resolver = app_handle.path_resolver();

	let node_path_opt = path_resolver.resolve_resource("bin/node");
	let script_path_opt = path_resolver.resolve_resource("scripts/cocoon/index.js");

	let (node_path_str, script_path_str) = match (node_path_opt, script_path_opt) {
		(Some(node_path), Some(script_path))
			if node_path.exists() && node_path.is_file() && script_path.exists() && script_path.is_file() =>
		{
			info!("[ProcMgmt] Using Node.js: {}", node_path.display());
			info!("[ProcMgmt] Using Cocoon script: {}", script_path.display());
			(
				node_path.to_string_lossy().into_owned(),
				script_path.to_string_lossy().into_owned(),
			)
		},
		(node_res_opt, script_res_opt) => {
			error!(
				"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Failed to resolve/validate paths. Node resolved: {}, Script \
				 resolved: {}. Node exists: {}, Script exists: {}. Ensure 'bin/node' and 'scripts/cocoon/index.js' \
				 are bundled and executable.",
				node_res_opt.is_some(),
				script_res_opt.is_some(),
				node_res_opt.as_ref().map_or(false, |p| p.exists()),
				script_res_opt.as_ref().map_or(false, |p| p.exists())
			);
			return;
		},
	};

	let mut command = Command::new(&node_path_str);
	command.arg(&script_path_str);
	command.arg(format!("--parent-pid={}", std::process::id()));
	// TODO: Add other args: --inspect-extensions, --logsPath, etc.
	command
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.kill_on_drop(true);

	info!(
		"[ProcMgmt] Spawning Cocoon: {} {} --parent-pid={}",
		node_path_str,
		script_path_str,
		std::process::id()
	);

	match command.spawn() {
		Ok(mut child_process) => {
			let child_pid_log_str = child_process.id().map_or_else(|| "unknown".to_string(), |pid| pid.to_string());
			info!("[ProcMgmt] Cocoon process spawned [OS PID: {}]", child_pid_log_str);

			if let Some(stderr_stream) = child_process.stderr.take() {
				let stderr_sidecar_id_clone = sidecar_id.clone();
				let stderr_pid_for_log_clone = child_pid_log_str.clone();
				tokio::spawn(async move {
					info!(
						"[Cocoon stderr ({})][PID: {}] Stderr monitoring started.",
						stderr_sidecar_id_clone, stderr_pid_for_log_clone
					);
					let reader = BufReader::new(stderr_stream);
					let mut lines = reader.lines();
					while let Ok(Some(line)) = lines.next_line().await {
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
				warn!("[ProcMgmt] Could not capture stderr for Cocoon '{}'.", sidecar_id);
			}

			match vine::setup_sidecar_communication(sidecar_id.clone(), child_process, app_handle.clone()) {
				Ok(_) => {
					info!(
						"[ProcMgmt] Vine IPC setup initiated for '{}' [OS PID: {}]",
						sidecar_id, child_pid_log_str
					);
					spawn_task_to_send_init_data_after_ready_signal(app_handle.clone(), sidecar_id.clone());
				},
				Err(vine_error) => {
					error!(
						"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Vine IPC setup failed for '{}' [OS PID: {}]: {}. Cocoon \
						 non-functional.",
						sidecar_id, child_pid_log_str, vine_error
					);
				},
			}
		},
		Err(spawn_error) => {
			error!(
				"[ProcMgmt] CRITICAL_STARTUP_FAILURE: Failed to spawn Cocoon using Node '{}'. Error: {}.",
				node_path_str, spawn_error
			);
		},
	}
}

fn spawn_task_to_send_init_data_after_ready_signal<R:Runtime>(app_handle:AppHandle<R>, sidecar_id:String) {
	tokio::spawn(async move {
		info!(
			"[ProcMgmt InitSender] Task for '{}' waiting for 'vine://sidecar/ready'...",
			sidecar_id
		);
		let (tx_ready_signal, mut rx_ready_signal) = tokio_mpsc::channel::<()>(1);
		let received_signal_flag = Arc::new(AtomicBool::new(false));
		let listener_app_handle = app_handle.clone();
		let listener_sidecar_id_clone = sidecar_id.clone();
		let listener_received_flag_clone = received_signal_flag.clone();

		let tauri_event_listener_id = listener_app_handle.listen_global("vine://sidecar/ready", move |event| {
			if let Some(payload_str) = event.payload() {
				match serde_json::from_str::<String>(payload_str) {
					Ok(ready_id_from_event) if ready_id_from_event == listener_sidecar_id_clone => {
						if listener_received_flag_clone
							.compare_exchange(false, true, AtomicOrdering::SeqCst, AtomicOrdering::SeqCst)
							.is_ok()
						{
							info!(
								"[ProcMgmt InitSender] Received 'vine://sidecar/ready' for '{}'.",
								listener_sidecar_id_clone
							);
							if tx_ready_signal.try_send(()).is_err() {
								error!(
									"[ProcMgmt InitSender] Failed internal ready send for '{}'.",
									listener_sidecar_id_clone
								);
							}
						}
					},
					Ok(other_id) => {
						trace!(
							"[ProcMgmt InitSender] 'vine://sidecar/ready' for different sidecar '{}' (expecting '{}').",
							other_id, listener_sidecar_id_clone
						);
					},
					Err(e) => {
						error!(
							"[ProcMgmt InitSender] Failed to parse 'vine://sidecar/ready' payload: {}. Payload: {}",
							e, payload_str
						);
					},
				}
			} else {
				warn!(
					"[ProcMgmt InitSender] 'vine://sidecar/ready' with no payload for '{}'.",
					listener_sidecar_id_clone
				);
			}
		});
		debug!(
			"[ProcMgmt InitSender] Listening for 'vine://sidecar/ready' from '{}' (Listener ID: {}).",
			sidecar_id, tauri_event_listener_id
		);

		let ready_timeout_duration = Duration::from_secs(30);
		match timeout(ready_timeout_duration, rx_ready_signal.recv()).await {
			Ok(Some(_)) => info!("[ProcMgmt InitSender] Ready confirm for '{}'. Sending initData.", sidecar_id),
			_ => {
				error!(
					"[ProcMgmt InitSender] No ready signal/timeout for '{}'. Aborting initData.",
					sidecar_id
				);
				app_handle.unlisten(tauri_event_listener_id);
				vine::unregister_sidecar_communication_channel(&sidecar_id); // Use the correct unregister function
				return;
			},
		}
		app_handle.unlisten(tauri_event_listener_id);

		info!(
			"[ProcMgmt InitSender] Sending main 'initExtensionHost' data to '{}'...",
			sidecar_id
		);
		let main_init_data_payload = construct_extension_host_init_data(&app_handle);
		if let Err(e) =
			vine::send_request_to_sidecar(&sidecar_id, "initExtensionHost".to_string(), main_init_data_payload, 60000)
				.await
		{
			error!(
				"[ProcMgmt InitSender] Failed 'initExtensionHost' request to '{}': {}. Sidecar may not init.",
				sidecar_id, e
			);
			vine::unregister_sidecar_communication_channel(&sidecar_id);
			return;
		} else {
			info!("[ProcMgmt InitSender] 'initExtensionHost' acknowledged by '{}'.", sidecar_id);
		}

		// --- Send FileSystem Provider Capabilities ---
		let mountain_env = app_handle.state::<Arc<AppRuntime>>().inner().get_environment(); // Get Arc<MountainEnvironment>
		let file_scheme_capabilities = mountain_env.get_file_provider_capabilities(); // Assuming this exists
		let file_uri_components_for_fs_info_rpc = json!({"scheme": "file", "path": "/", "$mid": 1});
		let fs_info_rpc_params = json!([file_uri_components_for_fs_info_rpc, file_scheme_capabilities]);
		let fs_info_rpc_method = format!("{}$acceptProviderInfos", ProxyTarget::ExtHostFileSystemInfo.target_prefix());
		info!(
			"[ProcMgmt InitSender] Sending FS caps (File: {}) to '{}' via '{}'.",
			file_scheme_capabilities, sidecar_id, fs_info_rpc_method
		);
		if let Err(e) = vine::send_request_to_sidecar(&sidecar_id, fs_info_rpc_method, fs_info_rpc_params, 5000).await {
			error!(
				"[ProcMgmt InitSender] Failed FS $acceptProviderInfos to '{}': {}",
				sidecar_id, e
			);
		} else {
			debug!(
				"[ProcMgmt InitSender] FS $acceptProviderInfos acknowledged by '{}'.",
				sidecar_id
			);
		}

		// --- Send Initial Full Configuration Model ---
		let app_state = app_handle.state::<AppState>();
		let config_init_data_dto_for_cocoon:IConfigurationInitDataDto = {
			let config_guard = app_state
				.configuration
				.lock()
				.expect("FATAL: Lock AppState.configuration for Cocoon config init failed.");
			let scopes_for_rpc = config_guard
				.get_all_configuration_scopes_for_rpc()
				.into_iter()
				.map(|(key, scope_enum)| (key, serde_json::to_value(scope_enum).unwrap_or(Value::Null)))
				.collect();
			IConfigurationInitDataDto {
				effective:config_guard.data.clone(),
				defaults:json!({"contents": {}}),
				user:json!({"contents": {}}),
				workspace:json!({"contents": {}}),
				folders:Value::Array(vec![]),
				memory:json!({"contents": {}}),
				policy:None,
				configuration_scopes:Some(scopes_for_rpc),
			}
		};
		let config_rpc_method =
			format!("{}$initializeConfiguration", ProxyTarget::ExtHostConfiguration.target_prefix());
		info!(
			"[ProcMgmt InitSender] Sending full config to '{}' via '{}'.",
			sidecar_id, config_rpc_method
		);
		if let Err(e) =
			vine::send_request_to_sidecar(&sidecar_id, config_rpc_method, json!(config_init_data_dto_for_cocoon), 10000)
				.await
		{
			error!(
				"[ProcMgmt InitSender] Failed full config $initializeConfiguration to '{}': {}",
				sidecar_id, e
			);
		} else {
			debug!(
				"[ProcMgmt InitSender] Full config $initializeConfiguration acknowledged by '{}'.",
				sidecar_id
			);
		}

		info!(
			"[ProcMgmt InitSender] All initial data sequences sent to Cocoon '{}'.",
			sidecar_id
		);
	});
}

fn construct_extension_host_init_data<R:Runtime>(app_handle:&AppHandle<R>) -> Value {
	let app_state = app_handle.state::<AppState>();
	let path_resolver = app_handle.path_resolver();
	let package_info = app_handle.package_info();
	info!("[ProcMgmt InitData] Constructing IExtensionHostInitData...");

	let path_to_uri_components_dto = |p:PathBuf, is_dir:bool| -> Value {
		let url_res = if is_dir { Url::from_directory_path(&p) } else { Url::from_file_path(&p) };
		let url = url_res.unwrap_or_else(|_| {
			Url::parse(&format!("file:///{}", p.to_string_lossy().replace('\\', "/")))
				.expect("Fallback URL parse failed")
		});
		json!({"scheme": url.scheme(), "authority": url.host_str().unwrap_or(""), "path": url.path(), "query": url.query().map(String::from), "fragment": url.fragment().map(String::from), "external": url.to_string(), "$mid": 1 })
	};

	let (
		workspace_id_str,
		workspace_name_str,
		workspace_config_uri_dto,
		workspace_folders_dto_vec,
		is_transient_bool,
		is_untitled_bool,
	) = {
		let folders_guard = app_state.workspace_folders.lock().expect("Lock workspace_folders failed");
		let config_path_guard = app_state
			.workspace_config_path
			.lock()
			.expect("Lock workspace_config_path failed");
		let id = app_state
			.get_workspace_id_string()
			.unwrap_or_else(|_| "FALLBACK_WORKSPACE_ID".to_string());
		let name = app_state
			.get_workspace_name()
			.unwrap_or_else(|_| "Untitled Workspace".to_string());
		let transient = false;
		let untitled = config_path_guard.is_none()
			&& folders_guard.len() <= 1
			&& (folders_guard.first().map_or(true, |f| f.uri.scheme() == "untitled"));
		let config_components_opt = config_path_guard.as_ref().map(|p| path_to_uri_components_dto(p.clone(), false));
		let folders_components_vec: Vec<Value> = folders_guard.iter().map(|f_state| {
            let f_uri_comp_dto = json!({"scheme": f_state.uri.scheme(), "authority": f_state.uri.host_str().unwrap_or(""), "path": f_state.uri.path(), "query": f_state.uri.query().map(String::from), "fragment": f_state.uri.fragment().map(String::from), "external": f_state.uri.to_string(), "$mid": 1 });
            json!({ "uri": f_uri_comp_dto, "name": f_state.name, "index": f_state.index })
        }).collect();
		(id, name, config_components_opt, folders_components_vec, transient, untitled)
	};
	let workspace_data_val = if workspace_folders_dto_vec.is_empty()
		&& workspace_config_uri_dto.is_none()
		&& !is_untitled_bool
	{
		Value::Null
	} else {
		json!({"id": workspace_id_str, "name": workspace_name_str, "configuration": workspace_config_uri_dto.unwrap_or(Value::Null), "folders": workspace_folders_dto_vec, "transient": is_transient_bool, "isUntitled": is_untitled_bool })
	};

	let (all_extensions_desc_dto_vec, my_extensions_ids_dto_vec, activation_events_map_dto) = {
		let scanned_extensions_guard = app_state.scanned_extensions.lock().expect("Lock scanned_extensions failed");
		let mut all_ext_descs:Vec<Value> = Vec::new();
		let mut my_ext_ids:Vec<Value> = Vec::new();
		let mut act_events:HashMap<String, Vec<String>> = HashMap::new();
		for (ext_full_id_str, ext_desc_state) in scanned_extensions_guard.iter() {
			match serde_json::to_value(ext_desc_state.clone()) {
				// Assuming ext_desc_state is ExtensionDescriptionState
				Ok(serialized_desc_dto) => {
					all_ext_descs.push(serialized_desc_dto);
					my_ext_ids.push(ext_desc_state.identifier.clone()); // identifier should be Value
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
					)
				},
			}
		}
		(all_ext_descs, my_ext_ids, act_events)
	};
	let extensions_snapshot_data_dto = json!({"versionId": 1, "allExtensions": all_extensions_desc_dto_vec, "myExtensions": my_extensions_ids_dto_vec, "activationEvents": activation_events_map_dto });

	let main_init_payload_config_data_dto:Value = {
		let config_guard = app_state
			.configuration
			.lock()
			.expect("Lock AppState.configuration for main initData failed.");
		let scopes_for_rpc:Vec<(String, Value)> = config_guard
			.get_all_configuration_scopes_for_rpc()
			.into_iter()
			.map(|(key, scope_enum)| (key, serde_json::to_value(scope_enum).unwrap_or(Value::Null)))
			.collect();
		json!({
			"effective": config_guard.data.clone(), "defaults": {"contents": {}}, "user": {"contents": {}},
			"workspace": {"contents": {}}, "folders": [], "memory": {"contents": {}}, "policy": Value::Null,
			"configurationScopes": scopes_for_rpc,
		})
	};

	let logs_loc_path = path_resolver
		.app_log_dir()
		.unwrap_or_else(|| PathBuf::from("./mountain_dev_logs_fallback"));
	let app_data_dir_base = path_resolver
		.app_data_dir()
		.unwrap_or_else(|| PathBuf::from("./mountain_dev_appdata_fallback"));
	let user_data_root_path = app_data_dir_base.join("User");
	let global_storage_home_path = user_data_root_path.join("globalStorage");
	let ws_storage_home_id_str = app_state
		.get_workspace_id_string()
		.unwrap_or_else(|_| "NO_WORKSPACE_ID_FOR_STORAGE_PATH".to_string());
	let workspace_storage_home_path = user_data_root_path.join("workspaceStorage").join(ws_storage_home_id_str);
	let app_root_heuristic_path = path_resolver
		.app_config_dir()
		.and_then(|p| p.parent().map(PathBuf::from))
		.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
	let enabled_proposed_apis_val =
		{ json!(*app_state.enabled_proposed_apis.lock().expect("Lock proposed APIs failed")) };
	let machine_id_str = app_handle.manager().instance_id().to_string();
	let session_id_str = Uuid::new_v4().to_string();
	let first_session_date_str = Utc::now().to_rfc3339();
	let current_language = std::env::var("LANG")
		.map(|s| s.split_once('.').map_or(s.as_str(), |(p, _)| p).to_lowercase())
		.unwrap_or_else(|_| "en".to_string());
	let product_config_dto_for_init_data = sky_dtos::ProductConfigurationDto {
		name_short:Some(package_info.name.chars().take(6).collect::<String>().to_uppercase()),
		name_long:Some(package_info.name.clone()),
		application_name:Some(package_info.name.to_lowercase().replace(' ', "-")),
		version:Some(package_info.version.to_string()),
		commit:Some(env!("CARGO_PKG_VERSION").to_string() + "-dev"), // Example
		date:Some(Utc::now().to_rfc3339()),
		data_folder_name:Some(format!(".{}", package_info.name.to_lowercase())),
		embedder_identifier:Some("desktop".to_string()),
		additional_properties:HashMap::new(),
	};

	json!({
		"commit": product_config_dto_for_init_data.commit.clone(), "version": product_config_dto_for_init_data.version.clone(),
		"quality": "development", "parentPid": std::process::id(),
		"environment": {
			"isExtensionDevelopmentDebug": false, "appName": package_info.name.clone(), "appHost": "desktop",
			"appRoot": path_to_uri_components_dto(app_root_heuristic_path, true),
			"appLanguage": current_language, "isExtensionTelemetryLoggingOnly": true,
			"appUriScheme": package_info.name.to_lowercase().replace(' ', "-"),
			"extensionDevelopmentLocationURI": [], "extensionTestsLocationURI": Value::Null,
			"globalStorageHome": path_to_uri_components_dto(global_storage_home_path, true),
			"workspaceStorageHome": path_to_uri_components_dto(workspace_storage_home_path, true),
			"useHostProxy": false, "skipWorkspaceStorageLock": true,
			"extensionEnabledProposedApi": enabled_proposed_apis_val,
		},
		"workspace": workspace_data_val,
		"remote": {"isRemote": false, "authority": Value::Null, "connectionData": Value::Null},
		"logLevel": map_log_level_filter_to_vscode_level(log::max_level()),
		"logsLocation": path_to_uri_components_dto(logs_loc_path, true),
		"loggers": [], "autoStart": true, "extensions": extensions_snapshot_data_dto,
		"nlsBaseUrl": Value::Null, "configurationData": main_init_payload_config_data_dto,
		"telemetryInfo": {
			"sessionId": session_id_str, "machineId": machine_id_str.clone(),
			"sqmId": machine_id_str.clone(), "devDeviceId": machine_id_str,
			"firstSessionDate": first_session_date_str,
		},
		"os": if cfg!(target_os = "windows") { 1 } else if cfg!(target_os = "macos") { 2 } else if cfg!(target_os = "linux") { 3 } else { 0 },
		"arch": std::env::consts::ARCH.to_string(),
		"product": product_config_dto_for_init_data,
		"uiKind": 1,
	})
}

fn map_log_level_filter_to_vscode_level(filter:LogLevelFilter) -> u32 {
	match filter {
		LogLevelFilter::Trace => 0,
		LogLevelFilter::Debug => 1,
		LogLevelFilter::Info => 2,
		LogLevelFilter::Warn => 3,
		LogLevelFilter::Error => 4,
		LogLevelFilter::Off => 6,
	}
}

// MountainEnvironment needs get_file_provider_capabilities()
// Example stub, actual implementation might involve querying FsConfig or
// capabilities.
impl MountainEnvironment {
	pub fn get_file_provider_capabilities(&self) -> u32 {
		// Based on vs_platform_files::FileSystemProviderCapabilities
		// For a local FS that supports basic read/write, case sensitive, no atomic ops
		// by default.
		let mut caps = 0;
		caps |= 1 << 1; // FileReadWrite
		caps |= 1 << 10; // PathCaseSensitive
		// Add other capabilities as supported by your FsReader/FsWriter implementation.
		// e.g., FileOpenReadWriteLock (1 << 2), FileFolderCopy (1 << 3),
		// FileWriteUnlock (1 << 5) For MVP, basic read/write and case sensitivity are
		// good starting points. FileReadWrite = 2, FileOpenReadWriteLock = 4,
		// FileFolderCopy = 8, FileWriteUnlock = 32, PathCaseSensitive = 1024 Example:
		// 2 (FileReadWrite) | 1024 (PathCaseSensitive) = 1026
		FileSystemProviderCapabilities::FileReadWrite as u32 | FileSystemProviderCapabilities::PathCaseSensitive as u32
	}
}
