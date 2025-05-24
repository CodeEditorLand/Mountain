// ---------------------------------------------------------------------------------------------
// Mountain Main Entry Point (main.rs)
// --------------------------------------------------------------------------------------------
// Main entry point for the Land editor's backend (Mountain) using Tauri.
// Sets up the Tauri application, initializes and manages shared application
// state (`AppState`) and the core application logic runner (`AppRuntime`).
// Registers `Track` dispatcher for frontend commands and uses the Tauri setup
// hook for critical initializations: launching Cocoon, setting up IPC/RPC,
// custom URI protocols, loading initial configurations, and scanning
// extensions.
//
// Responsibilities:
// - Configure and build the Tauri application.
// - Initialize `AppState` (default) and `AppRuntime`.
// - In `.setup()`:
//   - Perform AppState initializations requiring AppHandle (paths, extension
// 	scan, config load).
//   - Register frontend command handlers (`Track` and UI response handlers).
//   - Launch sidecar processes (Cocoon) via `process_mgmt`.
//   - Initialize IPC/RPC layers (`Vine`/`rpc`).
//   - Register custom URI protocol handlers (`handlers::protocol`).
//   - (Conceptually) Start file watchers for live config reloads.
// - Run the main Tauri event loop.
//
// Key Interactions:
// - Uses Tauri APIs for application building, state management, event handling,
//   path resolution.
// - Manages `AppState` and `AppRuntime` via Tauri's `State<T>`.
// - Calls `handlers::process_mgmt::launch_and_manage_cocoon`.
// - Calls `rpc::setup_mountain_rpc_server`.
// - Registers `track::dispatch_command`,
//   `handlers::sky_ui_responses::sky_resolves_ui_request`, and
//   `handlers::protocol::handle_vscode_protocol`.
// --------------------------------------------------------------------------------------------

// Hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, sync::Arc};

use log::{debug, error, info, trace, warn};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window, Wry};

// --- Application Modules ---
// mod app_state;

// mod environment;

// Parent module for sub-handlers
// mod handlers;
// Centralized logger initialization
// mod logging_setup;
// mod mist;

// mod rpc;

// mod runtime;

// mod track;

// mod vine;
use crate::app_state::AppState;
use crate::{environment::MountainEnvironment, runtime::AppRuntime};

#[derive(Clone, serde::Serialize)]
struct GenericPayload {
	// Example, may not be used directly in main
	message:String,
}

#[tokio::main]
async fn main() {
	// Initialize logger as the very first step
	logging_setup::init_mountain_logger();

	info!("[Mountain Main] Starting up Land Editor (Mountain)...");

	// AppState::default() loads some initial state (e.g., global memento from fixed
	// path)
	let initial_app_state = AppState::default();

	tauri::Builder::default()
		// Manage the initially created AppState
		.manage(initial_app_state)
		// AppRuntime and MountainEnvironment will be created and managed inside .setup once AppHandle is available
		.setup(|app| {
			info!("[Mountain Setup] Tauri setup hook running.");

			let app_handle = app.handle();

			// --- Create and Manage Environment & Runtime with AppHandle ---
			let mountain_env_arc = Arc::new(MountainEnvironment::new(app_handle.clone()));

			let app_runtime_arc = Arc::new(AppRuntime::new(mountain_env_arc.clone()));

			// Manage the fully initialized AppRuntime
			app_handle.manage(app_runtime_arc.clone());

			info!("[Mountain Setup] MountainEnvironment and AppRuntime created and managed.");

			// --- AppState Post-Handle Initialization Task ---
			// This task performs initializations that require AppHandle and should run asynchronously.
			let post_setup_app_handle = app_handle.clone();

			tauri::async_runtime::spawn(async move {
				info!("[Mountain Setup Task] Starting AppState post-handle initialization...");

				// Get managed AppState
				let app_state = post_setup_app_handle.state::<AppState>();

				// 1. Resolve and set extension scan paths
				// Assumes AppState.extension_scan_paths is Arc<Mutex<Vec<PathBuf>>>
				{
					let mut resolved_scan_paths: Vec<PathBuf> = Vec::new();

					if let Some(builtin_ext_dir) = post_setup_app_handle.path_resolver().resolve_resource("extensions/builtin") {
						if builtin_ext_dir.is_dir() {
							info!("[Mountain Setup Task] Adding builtin extension scan path: {}", builtin_ext_dir.display());

							resolved_scan_paths.push(builtin_ext_dir);
						} else {
							warn!("[Mountain Setup Task] Resolved builtin extension path is not a directory or does not exist: {}", builtin_ext_dir.display());
						}
					} else {
						warn!("[Mountain Setup Task] Could not resolve 'extensions/builtin' resource path.");
					}

					if let Some(user_ext_base_dir) = post_setup_app_handle.path_resolver().app_data_dir() {
						let user_ext_dir = user_ext_base_dir.join("extensions");

						if user_ext_dir.is_dir() {
							info!("[Mountain Setup Task] Adding user extension scan path: {}", user_ext_dir.display());

							resolved_scan_paths.push(user_ext_dir);
						} else {
							trace!("[Mountain Setup Task] User extension path does not exist, skipping: {}", user_ext_dir.display());
						}
					} else {
						warn!("[Mountain Setup Task] Could not resolve app data directory for user extensions path.");
					}

					let mut scan_paths_guard = app_state.extension_scan_paths.lock().expect("Failed to lock extension_scan_paths for writing");

					scan_paths_guard.clear();

					scan_paths_guard.extend(resolved_scan_paths);

					debug!("[Mountain Setup Task] Final extension scan paths set in AppState: {:?}", scan_paths_guard);
				}

				// 2. Scan for extensions (now uses the paths set above)
				app_state.scan_extensions().await;

				// 3. Load/configure enabled proposed APIs (example)
				let mut proposed_apis_guard = app_state.enabled_proposed_apis.lock().expect("Failed to lock proposed APIs for init");

				// For MVP, could be hardcoded or from a simple config file Mountain loads
				proposed_apis_guard.insert("*".to_string(), vec!["testProposedApi".to_string(), "workspaceTrust".to_string()]);

				info!("[Mountain Setup Task] Enabled proposed APIs configured. Count: {}", proposed_apis_guard.len());

				// 4. Load initial merged configuration into AppState
				match handlers::config::load_and_merge_configurations_internal(&post_setup_app_handle, &app_state).await {
					Ok(merged_config_state) => {
						app_state.configuration.lock().expect("Failed to lock AppState.configuration for init load").update_from(merged_config_state);

						info!("[Mountain Setup Task] Initial merged configuration loaded into AppState.");
					}
					Err(e) => {
						error!("[Mountain Setup Task] CRITICAL: Failed to load initial merged configurations: {}", e);
					}
				}

				// 5. Update workspace memento path based on initial workspace (if any)
				if let Some(app_data_dir_for_memento) = post_setup_app_handle.path_resolver().app_data_dir() {
					debug!("[Mountain Setup Task] Attempting to initialize workspace memento path based on data dir: {}", app_data_dir_for_memento.display());

					if let Err(e) = app_state.update_workspace_memento_path(&app_data_dir_for_memento) {
						error!("[Mountain Setup Task] Failed to initialize workspace memento path: {}", e);
					}
				} else {
					warn!("[Mountain Setup Task] App data directory not available for workspace memento path init at this stage.");
				}

				info!("[Mountain Setup Task] AppState post-handle initialization complete.");
			});

			// --- Register Custom Protocol Handlers ---
			let protocol_handle_setup = app_handle.clone();

			app.protocol()
				.register("vscode", move |request| {
					debug!("[Mountain Setup Protocol] Received vscode:// request: {}", request.uri());

					handlers::protocol::handle_vscode_protocol(request, protocol_handle_setup.clone())
				})
				.expect("Failed to register vscode protocol");

			info!("[Mountain Setup] vscode:// protocol registered.");


			// --- Setup RPC Server (for Vine/Sidecar Communication) ---
			let rpc_runtime_clone_setup = app_handle.state::<Arc<AppRuntime>>().inner().clone();

			let rpc_app_handle_clone_setup = app_handle.clone();

			rpc::setup_mountain_rpc_server(rpc_app_handle_clone_setup, rpc_runtime_clone_setup);

			info!("[Mountain Setup] RPC Server handlers conceptually registered for Vine.");

			// --- Launch Cocoon Sidecar (Conditional) ---
			#[cfg(feature = "extension_host_cocoon")]
			{
				info!("[Mountain Setup] Cocoon feature enabled. Spawning launch_and_manage_cocoon task...");

				let cocoon_app_handle = app_handle.clone();

				tauri::async_runtime::spawn(async move {
					handlers::process_mgmt::launch_and_manage_cocoon(cocoon_app_handle).await;
				});
			}

			#[cfg(not(feature = "extension_host_cocoon"))]
			{
				info!("[Mountain Setup] Cocoon feature disabled.");
			}

			// --- Start Native Mist WebSocket Server (Conditional) ---
			#[cfg(feature = "mist_native")]
			{
				info!("[Mountain Setup] Native Mist feature enabled. Spawning start_websocket_server task...");

				let mist_app_handle = app_handle.clone();

				tauri::async_runtime::spawn(async move {
					if let Err(e) = crate::mist::start_websocket_server(mist_app_handle).await {
						error!("[Mist Server] Native WebSocket server failed to start: {}", e);
					}
				});
			}

			#[cfg(not(feature = "mist_native"))]
			{
				info!("[Mountain Setup] Native Mist feature disabled.");
			}


			// --- TODO: Start File Watchers for Configuration Files ---
			// Example:
			// let watcher_app_handle = app_handle.clone();
			// tauri::async_runtime::spawn(async move {
			//	if let Err(e) = handlers::config_watcher::start_watching_config_files(watcher_app_handle).await {
			//		error!("[Config Watcher] Failed to start: {}", e);
			//	}
			// });

			info!("[Mountain Setup] Conceptual: File watchers for config would start here.");

			info!("[Mountain Setup] Setup hook logic complete.");

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			// Main entry point for frontend commands
			track::dispatch_command,
			// Handler for UI responses from Sky
			handlers::sky_ui_responses::sky_resolves_ui_request
		])
		.on_window_event(|event| match event.event() {
			tauri::WindowEvent::CloseRequested { api, .. } => {
				info!("[Mountain WindowEvent] Close requested for window: {}", event.window().label());

				// TODO: Implement graceful shutdown:
				// 1. Notify Cocoon to deactivate extensions.
				// 2. Wait for Cocoon to signal completion or timeout.
				// 3. Save AppState (mementos, dirty files via DocumentProvider effects).
				// 4. Allow close.
				// To prevent immediate close
				// Example: api.prevent_close();
			}

			tauri::WindowEvent::Destroyed => {
				info!("[Mountain WindowEvent] Window destroyed: {}", event.window().label());
			}

			_ => {
				trace!("[Mountain WindowEvent] Other event on window '{}': {:?}", event.window().label(), event.event());
			}

		})
		.build(tauri::generate_context!())
		.expect("Error while building Mountain Tauri application")
		.run(|app_handle_run, event| match event {
			tauri::RunEvent::ExitRequested { api, .. } => {
				info!("[Mountain RunEvent] Application exit requested. Performing cleanup...");

				// TODO: Global cleanup before app exit:
				// Call this if async cleanup is needed
				// api.prevent_exit();
				// let handle_clone_exit = app_handle_run.clone();

				// tauri::async_runtime::spawn(async move {
				//	info!("[Mountain Exit] Signaling sidecars to terminate...");

				//	// Example: vine::broadcast_terminate_signal(&handle_clone_exit).await;

				// Give time for sidecars
				//	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
				//	info!("[Mountain Exit] Exiting application now.");

				// Exit after cleanup
				//	handle_clone_exit.exit(0);
				// });

				info!("[Mountain RunEvent] Application process is beginning to exit or handling exit request.");
			}

			tauri::RunEvent::Exit => {
				info!("[Mountain RunEvent] Application has fully exited.");
			}

			tauri::RunEvent::Ready => {
				info!("[Mountain RunEvent] Application is ready.");
			}

			_ => {
				trace!("[Mountain RunEvent] Other/unhandled run event: {:?}", event);
			}
		});

	info!("[Mountain Main] Application event loop finished or error occurred.");
}
