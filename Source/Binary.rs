//! # Mountain Binary Entry Point
//!
//! This file orchestrates the entire application lifecycle. It is responsible
//! for setting up logging, initializing the `Echo` scheduler, the core
//! `ApplicationState`, the `ApplicationRunTime`, the `Vine` gRPC server, the
//! `Cocoon` sidecar process, and the Tauri application window and event loop.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Echo::Scheduler::SchedulerBuilder::SchedulerBuilder;
use log::{error, info};
use tauri::{Manager, RunEvent};

use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	ProcessManagement::CocoonManagement::InitializeCocoon, // Placeholder for future logic
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Vine,
};

/// Initializes the application's logging infrastructure using `env_logger`.
/// In debug builds, it defaults to a more verbose log level.
fn InitializeLogging() {
	let LogLevel = if cfg!(debug_assertions) { "debug" } else { "info" };
	if std::env::var("RUST_LOG").is_err() {
		// This is unsafe but only runs once at startup.
		unsafe { std::env::set_var("RUST_LOG", LogLevel) };
	}

	#[cfg(debug_assertions)]
	{
		env_logger::Builder::new()
			.filter_level(log::LevelFilter::Trace)
			.format(|Buffer, Record| {
				use std::io::Write;

				use colored::Colorize;
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
}

/// The main asynchronous function that sets up and runs the application.
pub fn Fn() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Cannot build.")
		.block_on(async {
			InitializeLogging();
			info!("[Main] Starting Mountain application...");

			// 1. Create the high-performance scheduler from the `Echo` crate.
			let NumberOfWorkers = num_cpus::get().max(2);
			let Scheduler = SchedulerBuilder::Create().WithWorkerCount(NumberOfWorkers).Build();

			// We need an Arc<> to safely share the scheduler for shutdown handling.
			let SchedulerForShutdown = Arc::new(Scheduler);
			let SchedulerForRunTime = SchedulerForShutdown.clone();

			#[allow(unused_mut)]
			let mut Builder = tauri::Builder::default();

			#[cfg(any(windows, target_os = "linux"))]
			{
				Builder = Builder.any_thread();
			}

			Builder
				.manage(ApplicationState::default())
				.setup(move |Application| {
					info!("[Setup] Tauri setup hook initiated.");
					let ApplicationHandle = Application.handle().clone();

					// 2. Create the application Environment and the Echo-powered
					//    ApplicationRunTime.
					let Environment = Arc::new(MountainEnvironment::Create(ApplicationHandle.clone()));
					let RunTime = Arc::new(ApplicationRunTime::Create(SchedulerForRunTime, Environment));

					// 3. Manage the ApplicationRunTime in Tauri's state so it's accessible to all
					//    command handlers.
					ApplicationHandle.manage(RunTime);
					info!("[Setup] Echo scheduler and ApplicationRunTime created and managed.");

					// 4. Spawn a detached task for all post-setup initializations.
					// This allows the UI to load faster while the backend finishes starting up.
					let PostSetupApplicationHandle = ApplicationHandle.clone();
					tauri::async_runtime::spawn(async move {
						info!("[SetupTask] Starting post-setup initializations...");
						let _ApplicationState = PostSetupApplicationHandle.state::<ApplicationState>();

						// TODO: Re-integrate handler logic for these initializations.
						// Handler::Configuration::InitializeConfiguration(&PostSetupApplicationHandle,
						// &ApplicationState).await;
						// Handler::ExtensionManagement::InitializeScanPaths(&
						// PostSetupApplicationHandle, &ApplicationState).await; ApplicationState.
						// ScanExtensions(&PostSetupApplicationHandle).await;

						Vine::Server::Initialize::Initialize(
							PostSetupApplicationHandle.clone(),
							"[::1]:50051".to_string(),
						);
						InitializeCocoon(&PostSetupApplicationHandle).await;

						info!("[SetupTask] Post-setup initializations complete.");
					});

					Ok(())
				})
				.plugin(tauri_plugin_dialog::init())
				.invoke_handler(tauri::generate_handler![
					// TODO: Re-integrate all Tauri command handlers here.
					crate::Track::DispatchLogic::DispatchFrontendCommand,
				])
				.build(tauri::generate_context!())
				.expect("FATAL: Error while building Mountain Tauri application")
				.run(move |ApplicationHandle, Event| {
					if let RunEvent::ExitRequested { api, .. } = Event {
						info!("[RunEvent] Exit requested. Initiating graceful shutdown...");
						api.prevent_exit();
						let SchedulerHandle = SchedulerForShutdown.clone();
						let ApplicationHandleClone = ApplicationHandle.clone();

						// Spawn a new async task to handle the shutdown to avoid blocking the
						// Tauri event loop.
						tokio::spawn(async move {
							info!("[Shutdown] Shutting down Echo scheduler...");
							if let Ok(mut Scheduler) = Arc::try_unwrap(SchedulerHandle) {
								Scheduler.Stop().await;
							} else {
								error!("[Shutdown] Could not get exclusive access to scheduler for shutdown.");
							}
							info!("[Shutdown] Shutdown complete. Exiting application.");
							ApplicationHandleClone.exit(0);
						});
					}
				});

			info!("[Main] Mountain application has shut down.");
		});
}
