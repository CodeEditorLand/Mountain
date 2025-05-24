// ---------------------------------------------------------------------------------------------
// Mountain Main Entry Point (main.rs)
// --------------------------------------------------------------------------------------------
// Main entry point for the Land editor's backend (Mountain) using Tauri.
// This file is responsible for:
// - Configuring and building the Tauri application.
// - Initializing and managing shared application state (`AppState`) and the
//   core application logic runner (`AppRuntime`). `AppState` is created with
//   default values first, and then `AppRuntime` (which depends on `AppHandle`
//   for its `MountainEnvironment`) is created and managed within the Tauri
//   `.setup()` hook.
// - Performing critical initializations within the `.setup()` hook once the
//   `AppHandle` is available. This includes:
//   - Finalizing `AppState` initialization (e.g., resolving extension scan
//     paths, loading initial merged configuration, scanning for extensions).
//     This is done in an async task to avoid blocking the setup hook.
//   - Registering Tauri command handlers, primarily `track::dispatch_command`
//     for frontend actions and
//     `handlers::sky_ui_responses::sky_resolves_ui_request` for UI callbacks
//     from Sky.
//   - Launching sidecar processes (e.g., Cocoon Node.js extension host) via
//     `handlers::process_mgmt::launch_and_manage_cocoon_sidecar`.
//   - Setting up the RPC server endpoint logic within `rpc.rs` (conceptually,
//     as `rpc.rs` provides handler structs used by `track.rs`).
//   - Registering custom URI protocol handlers (e.g., `vscode://` or
//     `landcode://`) via
//     `handlers::protocol::handle_custom_uri_scheme_request`.
//   - Optionally starting a native WebSocket server (Mist) if the `mist_native`
//     feature is enabled.
//   - (Conceptually) Starting file watchers for live configuration reloads.
// - Handling window events (e.g., `CloseRequested`, `Destroyed`).
// - Handling application lifecycle events (e.g., `ExitRequested`, `Ready`).
// - Running the main Tauri event loop.
//
// Key Interactions:
// - Uses Tauri APIs for application building, state management, event handling,
//   and path resolution.
// - Manages `AppState` (initialized with `AppState::default()`) and
//   `AppRuntime` (created in `.setup()`) via Tauri's `State<T>` mechanism.
// - Spawns asynchronous tasks for long-running initializations or background
//   processes like sidecar management and WebSocket servers.
// - Registers `track::dispatch_command` as the main entry point for commands
//   invoked from the Sky frontend.
// - Integrates with various handler modules (`handlers::*`), `rpc.rs`,
//   `track.rs`, and `vine.rs`.
// --------------------------------------------------------------------------------------------

// Hide console window on Windows in release builds for a cleaner user
// experience.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Standard library imports
use std::{path::PathBuf, sync::Arc};

// Logging facade
use log::{debug, error, info, trace, warn};
// Tauri essentials
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window, Wry};

// --- Application Modules ---
// These would typically be `mod module_name;` declarations if in a library crate,
// or `use crate::module_name;` if in a binary crate with modules in `src/`.
// For this synthesis, assuming they are accessible via `crate::...`
// mod app_state;

// mod environment;

// Parent module for various sub-handlers */
// mod handlers;

// Centralized logger initialization */
// mod logging_setup;

// Native WebSocket server (optional) */
// mod mist;

// RPC handler implementations */
// mod rpc;

// AppRuntime definition */
// mod runtime;

// Command dispatcher */
// mod track;

// Sidecar IPC layer */
// mod vine;

// Core application state
use crate::app_state::AppState;
// Concrete Environment implementation
use crate::environment::MountainEnvironment;
// Application's effect runner
use crate::runtime::AppRuntime;

// Used for Tauri event payloads if needed, though often specific structs are
// better.
#[derive(Clone, serde::Serialize)]
struct GenericTauriPayload {
	message:String,
}

// Main function uses Tokio multi-threaded runtime.
#[tokio::main]
async fn main() {
	// 1. Initialize the logger as the very first step. This ensures all subsequent
	//    startup messages are captured. `logging_setup::init_mountain_logger()`
	//    should configure `env_logger` or similar.
	logging_setup::init_mountain_logger();

	info!("[Mountain Main] Starting up Land Editor (Mountain backend)...");

	// 2. Create the initial `AppState`. `AppState::default()` loads some
	//    preliminary state (e.g., global memento from a fixed path, registers
	//    native commands). Further initialization requiring `AppHandle` (like path
	//    resolution for extensions) happens in `.setup()`.
	let initial_app_state = AppState::default();

	// 3. Configure and build the Tauri application.
	tauri::Builder::default()
		// Manage the initially created AppState. It can be accessed in `.setup()` and
		// commands.
		.manage(initial_app_state)
		// The `AppRuntime` and its `MountainEnvironment` depend on `AppHandle` for
		// accessing `AppState` and Tauri path resolvers. Thus, they are created and
		// managed *inside* the `.setup()` hook once `AppHandle` is available.
		.setup(|app| {

			// This closure runs after the WebView2 environment is ready but before the
			// main window is created and shown.
			info!("[Mountain Setup Hook] Tauri setup hook executing...");
			
			/* Get the AppHandle for use in setup. */
			let app_handle = app.handle();
			
			// --- Create and Manage MountainEnvironment & AppRuntime ---
			// `MountainEnvironment` needs `AppHandle` to access `AppState` and Tauri APIs.
			let mountain_env_arc = Arc::new(MountainEnvironment::new(app_handle.clone()));
			
			// `AppRuntime` wraps `MountainEnvironment` to run effects.
			let app_runtime_arc = Arc::new(AppRuntime::new(mountain_env_arc.clone()));
			
			// Make `AppRuntime` available as managed state for command handlers and other parts.
			app_handle.manage(app_runtime_arc.clone());
			
			info!("[Mountain Setup Hook] MountainEnvironment and AppRuntime created and managed.");
			
			// --- Asynchronous AppState Post-Handle Initialization ---
			// Spawn an async task for initializations that require `AppHandle` and might
			// involve I/O (e.g., scanning extensions, loading configs). This avoids
			// blocking the main setup thread.
			let post_setup_app_handle_clone = app_handle.clone();
			
			tauri::async_runtime::spawn(async move {

                info!("[Mountain Setup Task] Starting AppState post-AppHandle initialization...");
				
				/* Get managed AppState */
				let app_state = post_setup_app_handle_clone.state::<AppState>();
				
				/* 1. Resolve and set extension scan paths in AppState. */
                {
                    let mut resolved_scan_paths_vec: Vec<PathBuf> = Vec::new();
					
                    let path_resolver = post_setup_app_handle_clone.path_resolver();
					
					/* Bundled "builtin" extensions path */
                    if let Some(builtin_ext_dir_path) = path_resolver.resolve_resource("extensions/builtin") {
                        if builtin_ext_dir_path.is_dir() {
                            info!("[Mountain Setup Task] Adding builtin extension scan path: {}", builtin_ext_dir_path.display());
							
                            resolved_scan_paths_vec.push(builtin_ext_dir_path);
							
                        } else {
                            warn!("[Mountain Setup Task] Resolved builtin extension path '{}' is not a directory or does not exist. Skipping.", builtin_ext_dir_path.display());
							
                        }
						
                    } else {
                        warn!("[Mountain Setup Task] Could not resolve 'extensions/builtin' resource path for scanning.");
						
                    }
					
					/* User-installed extensions path (e.g., in app data directory) */
                    if let Some(app_data_dir_path) = path_resolver.app_data_dir() {
                        let user_ext_dir_path = app_data_dir_path.join("extensions");
						
						/* Only add if it exists */
						if user_ext_dir_path.is_dir() {
                            info!("[Mountain Setup Task] Adding user extension scan path: {}", user_ext_dir_path.display());
							
                            resolved_scan_paths_vec.push(user_ext_dir_path);
							
                        } else {
                            trace!("[Mountain Setup Task] User extension path '{}' does not exist. Skipping.", user_ext_dir_path.display());
							
                        }
						
                    } else {
                        warn!("[Mountain Setup Task] Could not resolve app data directory for user extensions path.");
						
                    }
					
					/* TODO: Add support for additional extension scan paths from configuration or CLI arguments. */
					
                    let mut scan_paths_guard = app_state.extension_scan_paths.lock()
                        .expect("FATAL: Failed to lock AppState.extension_scan_paths for writing during setup.");
					
                    scan_paths_guard.clear();
					
                    scan_paths_guard.extend(resolved_scan_paths_vec);
					
                    debug!("[Mountain Setup Task] Final extension scan paths set in AppState: {:?}", *scan_paths_guard);
					
                }
				
				/* 2. Scan for extensions using the paths now set in AppState. */
				/*    `scan_extensions_and_populate_state` reads package.json files and updates `AppState.scanned_extensions`. */
                app_state.scan_extensions_and_populate_state().await;
				
				/* 3. Configure enabled proposed APIs (example). */
				/*    In a real app, this might come from a settings file or developer flags. */
                let mut proposed_apis_guard = app_state.enabled_proposed_apis.lock()
                    .expect("FATAL: Failed to lock AppState.enabled_proposed_apis for init during setup.");
				
				/* Enable "testProposedApi" and "workspaceTrust" for all extensions ("*") as an example. */
                proposed_apis_guard.insert("*".to_string(), vec!["testProposedApi".to_string(), "workspaceTrust".to_string()]);
				
                info!("[Mountain Setup Task] Enabled proposed APIs configured. Count: {}", proposed_apis_guard.len());
				
				/* TODO: Implement a more sophisticated way to manage proposed API enablement per extension. */
				
				/* 4. Load initial merged configuration into AppState. */
				/*    This reads user, workspace, and folder settings.json files and merges them. */
                match handlers::config::load_and_merge_configurations_internal(&post_setup_app_handle_clone, &app_state).await {
                    Ok(merged_config_state) => {
                        app_state.configuration.lock()
                            .expect("FATAL: Failed to lock AppState.configuration for init load during setup.")
                            .update_from_new_state(merged_config_state);
						
                        info!("[Mountain Setup Task] Initial merged configuration loaded into AppState.");
						
                    }
					
                    Err(e) => {
                        error!("[Mountain Setup Task] CRITICAL_ERROR: Failed to load initial merged configurations: {}. Application may not function correctly.", e);
						
						/* TODO: Consider how to handle this failure. Default config? Notify user? */
                    }
					
                }
				
				/* 5. Update workspace memento path based on initial workspace (if any) and app data dir. */
                if let Some(app_data_dir_for_memento) = post_setup_app_handle_clone.path_resolver().app_data_dir() {
                    debug!("[Mountain Setup Task] Attempting to initialize workspace memento path using app data dir: {}", app_data_dir_for_memento.display());
					
                    if let Err(e) = app_state.update_workspace_memento_path_and_reload(&app_data_dir_for_memento) {
                        error!("[Mountain Setup Task] Failed to initialize workspace memento path and load data: {}", e);
						
                    }
					
                } else {
                    warn!("[Mountain Setup Task] App data directory not available; workspace memento path cannot be initialized at this stage.");
					
                }
				
                info!("[Mountain Setup Task] AppState post-AppHandle initialization complete.");
				
            });
			
			// --- Register Custom Protocol Handlers (e.g., vscode://) ---
			/* Clone AppHandle for the protocol closure. */
			let protocol_app_handle_clone = app_handle.clone();
			
			/* TODO: Make the custom scheme ("vscode", "landcode") configurable. */
			if let Err(e) = app
				.protocol()
				.register("vscode", move |ipc_request| {
					// This closure is called by Tauri when an OS-level URI request for "vscode://" is received.
					debug!(
						"[Mountain Setup Protocol CB] Received vscode:// URI request: {}",
						ipc_request.uri()
					);
					
					// Delegate to the specific handler function.
					handlers::protocol::handle_custom_uri_scheme_request(
						ipc_request,
						protocol_app_handle_clone.clone(),
					)
				}) {
				error!("[Mountain Setup Hook] CRITICAL_ERROR: Failed to register 'vscode://' custom URI protocol handler: {}", e);
				
				// This is a significant failure if deep linking or auth callbacks are needed.
			} else {
				info!("[Mountain Setup Hook] 'vscode://' custom URI protocol handler registered successfully.");
				
			}
			
			// --- Setup RPC Server Endpoint Logic (for Vine/Sidecar Communication) ---
			// `rpc::setup_mountain_rpc_server` currently logs a message. The actual RPC
			// method implementations are in `rpc.rs` and are invoked by `track.rs`.
			let rpc_setup_runtime_clone = app_handle.state::<Arc<AppRuntime>>().inner().clone();
			
			let rpc_setup_app_handle_clone = app_handle.clone();
			
			rpc::setup_mountain_rpc_server(rpc_setup_app_handle_clone, rpc_setup_runtime_clone);
			
			info!("[Mountain Setup Hook] RPC Server endpoint logic (via Track dispatcher) conceptually ready for Vine.");
			
			// --- Launch Cocoon Sidecar Process (conditional on feature flag) ---
			#[cfg(feature = "extension_host_cocoon")]
			{
				info!("[Mountain Setup Hook] 'extension_host_cocoon' feature IS ENABLED. Spawning Cocoon sidecar management task...");
				
				let cocoon_launch_app_handle_clone = app_handle.clone();
				
				tauri::async_runtime::spawn(async move {
					handlers::process_mgmt::launch_and_manage_cocoon_sidecar(
						cocoon_launch_app_handle_clone,
					)
					.await;
				
				});
				
			}
			
			#[cfg(not(feature = "extension_host_cocoon"))]
			{
				info!("[Mountain Setup Hook] 'extension_host_cocoon' feature is DISABLED. Cocoon sidecar will not be launched.");
				
			}
			
			// --- Start Native Mist WebSocket Server (conditional on feature flag) ---
			#[cfg(feature = "mist_native")]
			{
				info!("[Mountain Setup Hook] 'mist_native' feature IS ENABLED. Spawning native WebSocket server (Mist) task...");
				
				let mist_server_app_handle_clone = app_handle.clone();
				
				tauri::async_runtime::spawn(async move {
					if let Err(e) =
						crate::mist::start_websocket_server(mist_server_app_handle_clone).await
					{
						error!(
							"[Mist Server Startup] Native WebSocket server (Mist) failed to start: {}",
							e
						);
						
					}
					
				});
				
			}
			
			#[cfg(not(feature = "mist_native"))]
			{
				info!("[Mountain Setup Hook] 'mist_native' feature is DISABLED. Native WebSocket server (Mist) will not be started.");
				
			}
			
			// --- TODO: Start File Watchers for Configuration Files ---
			// This would involve using a crate like `notify` to watch `settings.json` files
			// (user, workspace, folder) and trigger `handlers::config::load_and_merge_configurations_internal`
			// and `handlers::config::notify_config_changed_for_keys` on changes.
			// Example conceptual placeholder:
			// let config_watcher_app_handle = app_handle.clone();
			
			// tauri::async_runtime::spawn(async move {
			//     if let Err(e) = handlers::config_watcher::start_watching_config_files(config_watcher_app_handle).await {
			//         error!("[Config Watcher Startup] Failed to start configuration file watcher: {}", e);
			
			//     }
			
			// });
			
			info!("[Mountain Setup Hook] Conceptual: File watchers for live configuration reload would start here.");
			
			info!("[Mountain Setup Hook] Setup hook logic completed.");
			
			Ok(())
		})
		// Register Tauri command handlers.
		.invoke_handler(tauri::generate_handler![
			// Main entry point for commands invoked from the Sky frontend.
			track::dispatch_command,
			// Handler for Sky frontend to send back results of UI interactions (dialogs, etc.).
			handlers::sky_ui_responses::sky_resolves_ui_request
			// TODO: Add other specific Tauri commands here if needed (e.g., for direct AppState queries not fitting the effect model).
		])
		// Handle window-specific events.
		.on_window_event(|event| match event.event() {
			tauri::WindowEvent::CloseRequested { api, .. } => {
				info!(
					"[Mountain WindowEvent] Close requested for window: {}",
					event.window().label()
				);
				
				// TODO: Implement graceful shutdown logic before allowing window close:
				// 1. Notify Cocoon (and other sidecars) to deactivate extensions and save state.
				//    - Send a specific Vine notification (e.g., "$shutdown").
				// 2. Wait for sidecars to acknowledge shutdown or timeout.
				// 3. Run `workspace_effects::save_all(true)` or similar to save dirty files.
				//    This needs to be done carefully, potentially prompting the user.
				// 4. Persist `AppState` mementos if they haven't been saved recently.
				// 5. After all cleanup, allow the close: `event.window().close()`.
				//
				// To prevent immediate close while async cleanup happens:
				// api.prevent_close();
				
				// let window_clone = event.window().clone();
				
				// tauri::async_runtime::spawn(async move {
				/* Perform async cleanup... */
				//    
				//     info!("[Graceful Shutdown] Async cleanup for window '{}' complete.", window_clone.label());
				
				//     if let Err(e) = window_clone.close() {
				//         error!("[Graceful Shutdown] Error closing window '{}' after cleanup: {}", window_clone.label(), e);
				
				//     }
				
				// });
				
				warn!("[Mountain WindowEvent] CloseRequested: Graceful shutdown logic is not fully implemented. Allowing default close behavior.");
				
			}
			
			tauri::WindowEvent::Destroyed => {
				info!(
					"[Mountain WindowEvent] Window destroyed: {}",
					event.window().label()
				);
				
				// If this is the main window, the application will typically exit soon.
			}
			
			_ => {
				// Log other window events at trace level to reduce noise.
				trace!(
					"[Mountain WindowEvent] Other event on window '{}': {:?}",
					event.window().label(),
					event.event()
				);
				
			}
			
		})
		// Build the Tauri application.
		/* Uses tauri.conf.json context */
		.build(tauri::generate_context!())
		.expect("FATAL: Error while building Mountain Tauri application")
		// Run the Tauri application event loop.
		.run(|app_handle_run, event| match event {
			tauri::RunEvent::ExitRequested { api, .. } => {
				info!(
					"[Mountain RunEvent] Application exit requested. Performing pre-exit cleanup..."
				);
				
				// This event is triggered before the app fully exits, allowing for cleanup.
				// Similar to WindowEvent::CloseRequested but for the whole app.
				// TODO: Implement global cleanup:
				//       - Ensure all sidecars are terminated gracefully.
				//       - Ensure all critical state (mementos, settings) is flushed to disk.
				//
				// Example for preventing immediate exit for async cleanup:
				// api.prevent_exit();
				
				// let app_handle_clone_exit = app_handle_run.clone();
				
				// tauri::async_runtime::spawn(async move {
				//     info!("[Mountain Exit Cleanup] Signaling sidecars to terminate...");
				
				/* Example: vine::broadcast_signal_to_all_sidecars(&app_handle_clone_exit, "$terminate", Value::Null).await; */
				//    
				/* Give sidecars time */
				//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
				
				//     info!("[Mountain Exit Cleanup] Persisting final application state...");
				
				/* Example: app_handle_clone_exit.state::<AppState>().save_all_mementos().await; */
				//    
				//     info!("[Mountain Exit Cleanup] Exiting application now.");
				
				/* Exit with success code */
				//     app_handle_clone_exit.exit(0);
				
				// });
				
				warn!("[Mountain RunEvent] ExitRequested: Global pre-exit cleanup is not fully implemented. Proceeding with default exit.");
				
			}
			
			tauri::RunEvent::Exit => {
				info!("[Mountain RunEvent] Application has fully exited.");
				
			}
			
			tauri::RunEvent::Ready => {
				info!("[Mountain RunEvent] Application is ready and main window (if any) should be visible.");
				
				// This is a good place for actions that need to happen after UI is up,
				// but before user interaction, if not handled by frontend.
			}
			
			// Log other run events at trace level.
			_ => {
				trace!("[Mountain RunEvent] Other/unhandled run event: {:?}", event);
				
			}
			
		});

	info!("[Mountain Main] Application event loop finished or an error occurred during run.");
}

// Define a minimal logging_setup module for completeness of main.rs compilation
mod logging_setup {
	pub fn init_mountain_logger() {
		// Basic env_logger setup, can be customized further.
		// Example: RUST_LOG=info,land_mountain=trace cargo run
		env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
			/* Optional: more precise timestamps */
			.format_timestamp_millis()
			.init();

		log::info!("[Logger Init] Mountain logger initialized via env_logger.");
	}
}
