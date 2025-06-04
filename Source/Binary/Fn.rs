// ---------------------------------------------------------------------------------------------
// Mountain Application Main Entry Point (Fn.rs)
// --------------------------------------------------------------------------------------------
// This file serves as the primary entry point for the Mountain Tauri
// application.
//
// Responsibilities:
// 1. Logger Initialization (Debug Builds Only).
// 2. Tokio Runtime Setup for asynchronous operations.
// 3. Tauri Application Setup:
//    - Initializes and manages core application state (`AppState`).
//    - Initializes the application runtime (`AppRuntime`) which holds the
//      `MountainEnvironment` for effect execution.
//    - Creates and configures the main application window.
//    - Registers Tauri command handlers for frontend-backend communication.
//    - Performs initial setup tasks like loading configuration, scanning
//      extensions.
//    - (Conceptually) Manages the lifecycle of sidecar processes (e.g.,
//      Cocoon).
//    - (Conceptually) Initializes IPC mechanisms (e.g., Vine).
// 4. Tauri Application Execution.
//
// This setup integrates the core "Mountain" backend system with the Tauri
// frontend framework.
// ---------------------------------------------------------------------------------------------

#![allow(non_snake_case)] // For types like Builder, Window, etc.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

use std::sync::Arc;

use colored::Colorize; // For logger
// Logging
use log::{debug, error, info, warn}; // Added warn and debug
// Mountain Crate Modules
use mountain::{
	// Assuming `mountain` is the crate name
	app_state::{self, AppState},                        // Central application state
	environment::MountainEnvironment,                   // Concrete environment for effects
	handlers::{self, sky_ipc_bridge, sky_ui_responses}, // Tauri command handlers
	runtime::AppRuntime,                                // Runtime for executing effects
	track,                                              /* RPC/Track layer handlers
	                                                     * sky_commands, // Assuming this module exists for specific Sky->Mountain commands
	                                                     * vine, // Conceptual IPC layer */
};
// Tauri
use tauri::{AppHandle, Manager, RunEvent, SystemTray, SystemTrayEvent, WebviewWindowBuilder, Wry};

// Placeholder for sky_commands if not fully defined elsewhere yet
mod sky_commands {
	use tauri::AppHandle;

	use crate::AppState; // Assuming AppState is accessible
	// Stubs for commands mentioned in snippets
	#[tauri::command]
	pub async fn mountain_set_zoom_level(
		_app_handle:AppHandle,
		_level:f64,
		_app_state:tauri::State<'_, AppState>,
	) -> Result<(), String> {
		log::warn!("STUB: mountain_set_zoom_level called with level: {}", _level);
		Ok(())
	}
	#[tauri::command]
	pub async fn mountain_fetch_shell_env(
		_app_handle:AppHandle,
		_app_state:tauri::State<'_, AppState>,
	) -> Result<std::collections::HashMap<String, String>, String> {
		log::warn!("STUB: mountain_fetch_shell_env called");
		Ok(std::collections::HashMap::new())
	}
	#[tauri::command]
	pub async fn mountain_get_process_memory_info(
		_app_handle:AppHandle,
		_app_state:tauri::State<'_, AppState>,
	) -> Result<serde_json::Value, String> {
		log::warn!("STUB: mountain_get_process_memory_info called");
		Ok(serde_json::json!({ "rss": 0, "heapTotal": 0, "heapUsed": 0, "external": 0 }))
	}
}

pub fn Fn() {
	// --- Logger Initialization (Debug Builds Only) ---
	#[cfg(debug_assertions)]
	{
		env_logger::Builder::new()
            .filter_level(log::LevelFilter::Trace) // Use Trace for more detail in debug
            .format(|Buffer, Record| {
                use std::io::Write;
                writeln!(
                    Buffer,
                    "[{}] [{}]: {}",
                    "Mountain".red(),
                    match Record.level() {
                        log::Level::Error => "ERROR".red().bold(),
                        log::Level::Warn => "WARN".yellow().bold(),
                        log::Level::Info => "INFO".green(),
                        log::Level::Debug => "DEBUG".blue(),
                        log::Level::Trace => "TRACE".magenta(),
                    },
                    Record.args()
                )
            })
            .try_init()
            .expect("Failed to initialize env_logger. Another logger might be active.");
	}

	info!("Starting Mountain application...");

	// --- Tokio Runtime Setup and Application Execution ---
	tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime.")
        .block_on(async {
            // --- Tauri Application Builder Initialization ---
            let mut Builder = tauri::Builder::default();

            #[cfg(any(windows, target_os = "linux"))]
            {
                Builder = Builder.any_thread();
            }

            // --- Tauri Application Setup ---
            Builder
                .setup(|app| {
                    info!("Tauri setup hook initiated.");

                    // --- Initialize Core Application State and Runtime ---
                    info!("Initializing AppState...");
                    let initial_app_state = AppState::default(); // Heavy lifting done in AppState::default()
                    app.manage(initial_app_state);
                    info!("AppState initialized and managed.");

                    let app_handle_clone = app.handle().clone();
                    let mountain_env = Arc::new(MountainEnvironment::new(app_handle_clone.clone()));
                    info!("MountainEnvironment initialized.");

                    let app_runtime = Arc::new(AppRuntime::new(mountain_env.clone(), app_handle_clone.clone()));
                    app.manage(app_runtime.clone()); // Manage AppRuntime for effect execution
                    info!("AppRuntime initialized and managed.");

                    // --- Perform Post-AppHandle Initializations ---
                    // Example: Resolve extension scan paths and scan extensions
                    // This needs to be async if scan_extensions_and_populate_state is async
                    let app_state_for_init = app.state::<AppState>();
                    let app_handle_for_init = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        info!("Performing post-AppHandle initializations (config load, extension scan)...");
                        // 1. Resolve extension scan paths
                        let mut scan_paths = Vec::new();
                        if let Some(resource_dir) = app_handle_for_init.path_resolver().resolve_resource("extensions") {
                            if resource_dir.exists() {
                                info!("Adding bundled extensions path: {}", resource_dir.display());
                                scan_paths.push(resource_dir);
                            } else {
                                warn!("Bundled extensions path does not exist: {}", resource_dir.display());
                            }
                        }
                        // TODO: Add user extensions path, dev extensions path
                        *app_state_for_init.extension_scan_paths.lock().unwrap() = scan_paths;

                        // 2. Scan extensions
                        app_state_for_init.scan_extensions_and_populate_state().await;
                        info!("Extension scan complete.");

                        // 3. Load initial merged configuration
                        // This would typically involve AppRuntime or direct handlers::config calls
                        match handlers::config::load_and_merge_configurations_internal(&app_handle_for_init).await {
                            Ok(_) => info!("Initial configurations loaded and merged."),
                            Err(e) => error!("Failed to load initial configurations: {:?}", e),
                        }

                        // 4. Update workspace memento path (if workspace was opened from cmd line args)
                        // This needs the app_data_dir, which AppState figures out in default()
                        // but update_workspace_memento_path needs the app_data_dir again.
                        // For simplicity, assuming AppState.global_memento_path.parent() is app_data_dir/User
                        if let Some(user_data_dir) = app_state_for_init.global_memento_path.parent().and_then(|p| p.parent()) {
                             if let Err(e) = app_state_for_init.update_workspace_memento_path_and_reload(user_data_dir){
                                error!("Failed to update initial workspace memento path: {}", e);
                             }
                        } else {
                            error!("Could not determine app_data_dir for initial memento path update.");
                        }


                        // TODO: Initialize IPC (Vine) and start sidecar (Cocoon) processes.
                        // info!("Initializing IPC (Vine)...");
                        // info!("Starting sidecar (Cocoon) process...");

                        info!("Post-AppHandle initializations complete.");
                    });


                    // --- Main Window Configuration ---
                    info!("Configuring main application window...");
                    let mut window_builder = WebviewWindowBuilder::new(
                        app,
                        "Application", // Internal label
                        tauri::WebviewUrl::App(std::path::PathBuf::from("Application/index.html")),
                    )
                    .use_https_scheme(true)
                    .zoom_hotkeys_enabled(true)
                    .browser_extensions_enabled(false);

                    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                    {
                        window_builder = window_builder
                            .title("FIDDEE") // Window title
                            .maximized(true)
                            .decorations(false) // Custom title bar assumed in frontend
                            .shadow(true);
                    }

                    match window_builder.build() {
                        Ok(window_instance) => {
                            info!("Main application window created successfully.");
                            #[cfg(all(debug_assertions, not(any(target_os = "android", target_os = "ios"))))]
                            {
                                window_instance.open_devtools();
                                info!("Developer tools opened for main window.");
                            }
                        }
                        Err(e) => {
                            error!("Main application window build failed: {:?}", e);
                            panic!("Main application window build failed: {:?}", e);
                        }
                    };
                    Ok(())
                })
                .plugin(tauri_plugin_dialog::init()) // Dialog plugin
                // Add other plugins here if needed
                .invoke_handler(tauri::generate_handler![
                    // Core RPC dispatcher
                    track::dispatch_command,
                    // UI Interaction Callbacks
                    handlers::sky_ui_responses::sky_resolves_ui_request,
                    // Configuration
                    track::mountain_get_workbench_configuration,
                    // Window/Env Commands (from Sky)
                    sky_commands::mountain_set_zoom_level,
                    sky_commands::mountain_fetch_shell_env,
                    sky_commands::mountain_get_process_memory_info,
                    // IPC Bridge (Sky <-> Sidecars via Mountain)
                    sky_ipc_bridge::mountain_ipc_bridge_send,
                    sky_ipc_bridge::mountain_ipc_bridge_invoke,
                    // Language Features (from Sky)
                    track::mountain_request_hover,
                    track::mountain_request_completions,
                    track::mountain_resolve_completion_item
                    // Add other specific track handlers here as they are created
                ])
                .build(tauri::generate_context!()) // Build the app instance
                .expect("Error while building Tauri application instance.")
                .run(|_app_handle, event| match event { // Use _app_handle to avoid unused warning
                    RunEvent::ExitRequested { api, .. } => {
                        info!("Exit requested. Preventing default exit to allow cleanup.");
                        api.prevent_exit();
                        // TODO: Implement graceful shutdown logic here:
                        // - Notify sidecars to terminate.
                        // - Wait for sidecars to exit.
                        // - Save any pending state.
                        // - Then, call app_handle.exit(0).
                        // For now, just exit immediately.
                        warn!("Graceful shutdown not fully implemented. Exiting immediately.");
                        _app_handle.exit(0);
                    }
                    RunEvent::Updater(updater_event) => {
                        info!("Updater event: {:?}", updater_event);
                        // Handle updater events (e.g., show progress, ask for restart)
                    }
                    _ => {}
                });
        });

	info!("Mountain application has shut down.");
}
