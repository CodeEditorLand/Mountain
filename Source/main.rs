//! The main entry point for the Mountain backend.
//!
//! This file orchestrates the entire application lifecycle, including Tauri
//! setup, initialization of the `AppState`, `Echo` scheduler, `AppRuntime`,
//! gRPC server, and the `Cocoon` sidecar process.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod app_state;
mod environment;
mod handlers;
mod runtime;
mod scheduler;
mod track;
mod vine;

use std::sync::Arc;

use Echo::scheduler::SchedulerBuilder;
use log::info;
use tauri::{Manager, RunEvent};

use crate::{
	app_state::AppState::AppState,
	environment::MountainEnvironment,
	runtime::AppRuntime,
	scheduler::Scheduler, // This is likely an alias or re-export of Echo::Scheduler
};

fn InitializeLogging() {
	// In a real application, this would set up env_logger or a similar logging
	// facade.
	if std::env::var("RUST_LOG").is_err() {
		std::env::set_var("RUST_LOG", "info");
	}
	env_logger::init();
}

#[tokio::main]
async fn main() {
	InitializeLogging();
	info!("[Main] Starting Mountain Application...");

	// 1. Create the high-performance scheduler.
	let NumberOfWorker = num_cpus::get().max(2);
	let Scheduler = SchedulerBuilder::New().WithWorkerCount(NumberOfWorker).Build();

	// We need an Arc<Mutex<>> to safely share the scheduler for shutdown handling.
	let SchedulerForShutdown = Arc::new(tokio::sync::Mutex::new(Scheduler));
	let SchedulerForRuntime = SchedulerForShutdown.clone();

	let mut Builder = tauri::Builder::default();

	// Platform-specific builder logic would go here if needed.

	Builder
		.manage(AppState::default())
		.setup(move |App| {
			info!("[Setup] Tauri setup hook initiated.");
			let AppHandle = App.handle().clone();

			// 2. Create the application Environment and the Echo-powered AppRuntime.
			let Environment = Arc::new(MountainEnvironment::New(AppHandle.clone()));
			let Runtime = Arc::new(AppRuntime::New(SchedulerForRuntime, Environment));

			// 3. Manage the AppRuntime in Tauri's state so it's accessible everywhere.
			AppHandle.manage(Runtime);
			info!("[Setup] Echo scheduler and AppRuntime created and managed by Tauri.");

			// 4. Spawn a detached task for all post-setup initializations.
			// This allows the UI to load faster while the backend finishes starting up.
			let PostSetupAppHandle = AppHandle.clone();
			tauri::async_runtime::spawn(async move {
				info!("[SetupTask] Starting post-setup initializations...");
				let AppState = PostSetupAppHandle.state::<AppState>();

				// This initialization order is important.
				handlers::config::InitializeConfiguration(&PostSetupAppHandle, &AppState).await;
				handlers::extension_management::ScanExtension(&PostSetupAppHandle, &AppState).await;
				vine::server::Initialize(PostSetupAppHandle.clone(), "[::1]:50051".to_string());
				handlers::process_management::InitializeCocoon(&PostSetupAppHandle).await;

				info!("[SetupTask] Post-setup initializations complete.");
			});

			// 5. Register the custom URI scheme protocol.
			if let Err(e) = App.protocol().register("vscode", move |r| {
				handlers::protocol::HandleCustomUriSchemeRequest(r, AppHandle.clone())
			}) {
				log::error!("[Setup] CRITICAL: Failed to register 'vscode://' protocol: {}", e);
			}

			Ok(())
		})
		.plugin(tauri_plugin_dialog::init())
		.invoke_handler(tauri::generate_handler![
			// --- Core command dispatcher ---
			track::DispatchCommand,
			// --- Specific UI/Sky-related handlers ---
			handlers::sky_command::MountainFetchShellEnv,
			handlers::sky_command::MountainGetProcessMemoryInfo,
			handlers::sky_command::MountainSetZoomLevel,
			handlers::sky_ipc_bridge::MountainIpcBridgeInvoke,
			handlers::sky_ipc_bridge::MountainIpcBridgeSend,
			handlers::sky_ui_response::SkyResolveUiRequest,
			handlers::tree_view::MountainRequestTreeChild,
		])
		.build(tauri::generate_context!())
		.expect("FATAL: Error while building Mountain Tauri application")
		.run(move |_AppHandle, Event| {
			if let RunEvent::ExitRequested { api, .. } = Event {
				info!("[RunEvent] Exit requested. Initiating graceful shutdown...");
				api.prevent_exit();
				let SchedulerHandle = SchedulerForShutdown.clone();

				// Spawn a new blocking thread to run the async shutdown logic.
				// This avoids deadlocks when shutting down the tokio runtime from within
				// itself.
				std::thread::spawn(move || {
					let Runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
					Runtime.block_on(async move {
						let mut Scheduler = SchedulerHandle.lock().await;
						Scheduler.Shutdown().await;
					});
				});

				_AppHandle.exit(0);
			}
		});

	info!("[Main] Mountain application has shut down.");
}
