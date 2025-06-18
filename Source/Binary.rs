// @file Binary.rs
// @brief The main entry point for the Mountain backend.
//
// This file orchestrates the entire application lifecycle, including Tauri
// setup, initialization of the `ApplicationState`, `Echo` scheduler,
// `ApplicationRunTime`, gRPC server, and the `Cocoon` sidecar process.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use std::sync::Arc;

use echo::scheduler::SchedulerBuilder;
use log::{error, info};
use tauri::{Manager, RunEvent};

use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	Environment::MountainEnvironment,
	Handler,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track,
	Vine,
};

/// Initializes the application's logging infrastructure using `env_logger`.
/// In debug builds, it defaults to a more verbose level.
fn initialize_logging() {
	let log_level = if cfg!(debug_assertions) { "debug" } else { "info" };
	if std::env::var("RUST_LOG").is_err() {
		std::env::set_var("RUST_LOG", log_level);
	}
	env_logger::init();
}

/// The main asynchronous function that sets up and runs the application.
#[tokio::main]
pub async fn Fn() {
	initialize_logging();
	info!("[Main] Starting Mountain Application...");

	// 1. Create the high-performance scheduler from the `echo` crate.
	let number_of_workers = num_cpus::get().max(2);
	let scheduler = SchedulerBuilder::New().WithWorkerCount(number_of_workers).Build();

	// We need an Arc<> to safely share the scheduler for shutdown handling.
	let scheduler_for_shutdown = Arc::new(scheduler);
	let scheduler_for_runtime = scheduler_for_shutdown.clone();

	let mut builder = tauri::Builder::default();

	builder
		.manage(ApplicationState::default())
		.setup(move |app| {
			info!("[Setup] Tauri setup hook initiated.");
			let app_handle = app.handle().clone();

			// 2. Create the application Environment and the Echo-powered
			//    ApplicationRunTime.
			let Environment = Arc::new(MountainEnvironment::New(app_handle.clone()));
			let run_time = Arc::new(ApplicationRunTime::New(scheduler_for_runtime, Environment));

			// 3. Manage the ApplicationRunTime in Tauri's state so it's accessible
			//    everywhere.
			app_handle.manage(run_time);
			info!("[Setup] Echo scheduler and ApplicationRunTime created and managed by Tauri.");

			// 4. Spawn a detached task for all post-setup initializations.
			// This allows the UI to load faster while the backend finishes starting up.
			let post_setup_app_handle = app_handle.clone();
			tauri::async_runtime::spawn(async move {
				info!("[SetupTask] Starting post-setup initializations...");
				let app_state = post_setup_app_handle.state::<ApplicationState>();

				// This initialization order is important.
				Handler::config::InitializeConfiguration(&post_setup_app_handle, &app_state).await;
				Handler::extension_management::InitializeScanPaths(&post_setup_app_handle, &app_state).await;
				app_state.ScanExtensions(&post_setup_app_handle).await;
				Vine::server::Initialize(post_setup_app_handle.clone(), "[::1]:50051".to_string());
				Handler::process_management::InitializeCocoon(&post_setup_app_handle).await;

				info!("[SetupTask] Post-setup initializations complete.");
			});

			// 5. Register the custom URI scheme protocol.
			if let Err(e) = app.protocol().register("vscode", move |r| {
				Handler::protocol::HandleCustomUriSchemeRequest(r, app_handle.clone())
			}) {
				error!("[Setup] CRITICAL: Failed to register 'vscode://' protocol: {}", e);
			}

			Ok(())
		})
		.plugin(tauri_plugin_dialog::init())
		.invoke_handler(tauri::generate_handler![
			// --- Core command dispatcher ---
			Track::DispatchCommand,
			// --- Specific UserInterface/Sky-related Handlers ---
			Handler::sky_command::MountainFetchShellEnv,
			Handler::sky_command::MountainGetProcessMemoryInfo,
			Handler::sky_command::MountainSetZoomLevel,
			Handler::sky_IPC_bridge::MountainIpcBridgeInvoke,
			Handler::sky_IPC_bridge::MountainIpcBridgeSend,
			Handler::sky_user_interface_response::SkyResolvesUiRequest,
			Handler::tree_view::MountainRequestTreeChild,
		])
		.build(tauri::generate_context!())
		.expect("FATAL: Error while building Mountain Tauri application")
		.run(move |app_handle, event| {
			if let RunEvent::ExitRequested { api, .. } = event {
				info!("[RunEvent] Exit requested. Initiating graceful shutdown...");
				api.prevent_exit();
				let scheduler_handle = scheduler_for_shutdown.clone();

				// Spawn a new async task on the tokio runtime to handle the shutdown.
				// This avoids blocking the Tauri event loop.
				tokio::spawn(async move {
					info!("[Shutdown] Shutting down Echo scheduler...");
					// We need to get mutable access to the scheduler to call shutdown.
					// Arc::into_inner is the safest way to do this, ensuring no other references
					// exist.
					if let Ok(mut scheduler) = Arc::try_unwrap(scheduler_handle) {
						scheduler.Shutdown().await;
					} else {
						error!(
							"[Shutdown] Could not get exclusive access to scheduler for shutdown. It may already be \
							 shutting down."
						);
					}
					info!("[Shutdown] Shutdown complete. Exiting application.");
					app_handle.exit(0);
				});
			}
		});

	info!("[Main] Mountain application has shut down.");
}
