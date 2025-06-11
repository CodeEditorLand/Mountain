// The main entry point for the Mountain backend.
//
// This file orchestrates the entire application lifecycle, including Tauri
// setup, initialization of the `ApplicationState`, `Echo` scheduler,
// `ApplicationRunTime`, gRPC server, and the `Cocoon` sidecar process.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// mod Handler;
// mod RunTime;
// mod ApplicationState;
// mod Environment;
// mod Scheduler;
// mod Track;
// mod Vine;

use std::sync::Arc;

use Echo::scheduler::SchedulerBuilder;
use log::info;
use tauri::{Manager, RunEvent};

use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	Environment::MountainEnvironment,
	Handler,
	RunTime::ApplicationRunTime,
	Track::DispatchCommand,
	scheduler::Scheduler,
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
pub async fn Fn() {
	InitializeLogging();
	info!("[Main] Starting Mountain Application...");

	// 1. Create the high-performance scheduler.
	let NumberOfWorker = num_cpus::get().max(2);
	let Scheduler = SchedulerBuilder::New().WithWorkerCount(NumberOfWorker).Build();

	// We need an Arc<Mutex<>> to safely share the scheduler for shutdown handling.
	let SchedulerForShutDown = Arc::new(tokio::sync::Mutex::new(Scheduler));
	let SchedulerForRunTime = SchedulerForShutDown.clone();

	let mut Builder = tauri::Builder::default();

	// Platform-specific builder logic would go here if needed.

	Builder
		.manage(ApplicationState::default())
		.setup(move |App| {
			info!("[Setup] Tauri setup hook initiated.");
			let ApplicationHandle = App.handle().clone();

			// 2. Create the application Environment and the Echo-powered
			//    ApplicationRunTime.
			let Environment = Arc::new(MountainEnvironment::New(ApplicationHandle.clone()));
			let RunTime = Arc::new(ApplicationRunTime::New(SchedulerForRunTime, Environment));

			// 3. Manage the ApplicationRunTime in Tauri's state so it's accessible
			//    everywhere.
			ApplicationHandle.manage(RunTime);
			info!("[Setup] Echo scheduler and ApplicationRunTime created and managed by Tauri.");

			// 4. Spawn a detached task for all post-setup initializations.
			// This allows the UI to load faster while the backend finishes starting up.
			let PostSetupApplicationHandle = ApplicationHandle.clone();
			tauri::async_RunTime::spawn(async move {
				info!("[SetupTask] Starting post-setup initializations...");
				let ApplicationState = PostSetupApplicationHandle.state::<ApplicationState>();

				// This initialization order is important.
				Handler::config::InitializeConfiguration(&PostSetupApplicationHandle, &ApplicationState).await;
				Handler::extension_management::ScanExtension(&PostSetupApplicationHandle, &ApplicationState).await;
				vine::server::Initialize(PostSetupApplicationHandle.clone(), "[::1]:50051".to_string());
				Handler::process_management::InitializeCocoon(&PostSetupApplicationHandle).await;

				info!("[SetupTask] Post-setup initializations complete.");
			});

			// 5. Register the custom URI scheme protocol.
			if let Err(e) = App.protocol().register("vscode", move |r| {
				Handler::protocol::HandleCustomUriSchemeRequest(r, ApplicationHandle.clone())
			}) {
				log::error!("[Setup] CRITICAL: Failed to register 'vscode://' protocol: {}", e);
			}

			Ok(())
		})
		.plugin(tauri_plugin_dialog::init())
		.invoke_handler(tauri::generate_handler![
			// --- Core command dispatcher ---
			DispatchCommand,
			// --- Specific UserInterface/Sky-related Handler ---
			Handler::SkyCommand::MountainFetchShellEnv,
			Handler::SkyCommand::MountainGetProcessMemoryInfo,
			Handler::SkyCommand::MountainSetZoomLevel,
			Handler::SkyIPCBridge::MountainIpcBridgeInvoke,
			Handler::SkyIPCBridge::MountainIpcBridgeSend,
			Handler::SkyUserInterfaceResponse::SkyResolveUiRequest,
			Handler::tree_view::MountainRequestTreeChild,
		])
		.build(tauri::generate_context!())
		.expect("FATAL: Error while building Mountain Tauri application")
		.run(move |_ApplicationHandle, Event| {
			if let RunEvent::ExitRequested { api, .. } = Event {
				info!("[RunEvent] Exit requested. Initiating graceful shutdown...");
				api.prevent_exit();
				let SchedulerHandle = SchedulerForShutDown.clone();

				// Spawn a new blocking thread to run the async shutdown logic.
				// This avoids deadlocks when shutting down the tokio RunTime from within
				// itself.
				std::thread::spawn(move || {
					let RunTime = tokio::RunTime::Builder::new_current_thread().enable_all().build().unwrap();
					RunTime.block_on(async move {
						let mut Scheduler = SchedulerHandle.lock().await;
						Scheduler.ShutDown().await;
					});
				});

				_ApplicationHandle.exit(0);
			}
		});

	info!("[Main] Mountain application has shut down.");
}
