// File: Mountain/Source/Binary.rs
// Role: Main entry point for the Mountain native host application.
// Responsibilities:
//   - Orchestrate the entire application lifecycle.
//   - Initialize logging, the Echo scheduler, ApplicationState, and
//     ApplicationRunTime.
//   - Parse command-line arguments to open a workspace.
//   - Bootstrap native command registration.
//   - Set up the Vine gRPC server and spawn the Cocoon sidecar process.
//   - Create and customize the main Tauri application window.
//   - Manage the main application event loop and graceful shutdown.

//! # Mountain Binary Entry Point
//!
//! This file orchestrates the entire application lifecycle. It is responsible
//! for setting up logging, initializing the `Echo` scheduler, the core
//! `ApplicationState`, the `ApplicationRunTime`, the `Vine` gRPC server, the
//! `Cocoon` sidecar process, and the Tauri application window and event loop.

#![allow(non_snake_case, non_camel_case_types)]

use std::{
	path::PathBuf,
	sync::{Arc, Mutex},
};

use Echo::Scheduler::SchedulerBuilder::SchedulerBuilder;
use log::{error, info, warn};
use tauri::{AppHandle, Manager, RunEvent, Wry};

use crate::{
	ApplicationState::{
		ApplicationState::{ApplicationState, MapLockError},
		Internal::ScanAndPopulateExtensions,
	},
	Command,
	Environment::{ConfigurationProvider::InitializeAndMergeConfigurations, MountainEnvironment::MountainEnvironment},
	ProcessManagement::{CocoonManagement::InitializeCocoon, InitializationData},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Vine,
};

/// Initializes the application's logging infrastructure using `env_logger`.
fn InitializeLogging() {
	let LogLevel = if cfg!(debug_assertions) { "debug" } else { "info" };

	if std::env::var("RUST_LOG").is_err() {
		// Use of unsafe is not ideal, but this is a common pattern for setting
		// an env var for the current process only if it's not already set.
		// A safer alternative would involve re-architecting how logging is configured.
		unsafe { std::env::set_var("RUST_LOG", LogLevel) };
	}

	let mut Builder = env_logger::Builder::new();

	Builder.filter_level(log::LevelFilter::Trace);

	#[cfg(debug_assertions)]
	{
		Builder.format(|Buffer, Record| {
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
		});
	}

	if Builder.try_init().is_err() {
		warn!("[Main] Failed to initialize env_logger. Another logger might be active.");
	}
}

/// A Tauri command to provide the initial workbench configuration to the Sky
/// frontend.
#[tauri::command]
async fn MountainGetWorkbenchConfiguration(
	ApplicationHandle:AppHandle<Wry>,

	State:tauri::State<'_, Arc<ApplicationState>>,
) -> Result<serde_json::Value, String> {
	info!("[IPC Bridge] Received MountainGetWorkbenchConfiguration request from Sky.");

	InitializationData::ConstructSandboxConfiguration(&ApplicationHandle, &State)
		.await
		.map_err(|Error| Error.to_string())
}

/// The main asynchronous function that sets up and runs the application.
pub fn Fn() {
	// Create a dedicated multi-threaded Tokio runtime.
	let Runtime = tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Cannot build Tokio runtime.");

	Runtime.block_on(async {
		InitializeLogging();

		info!("[Main] Starting Mountain application...");

		// --- Pre-flight WorkSpace Loading from Args ---
		let CliArgs:Vec<String> = std::env::args().collect();

		let WorkSpacePathArgument = CliArgs.iter().find(|Arg| Arg.ends_with(".code-workspace"));

		let (InitialFolders, WorkSpaceConfigurationPath) = if let Some(PathString) = WorkSpacePathArgument {
			let Path = PathBuf::from(PathString);

			info!("[Main] Found workspace argument: {}", Path.display());

			match std::fs::read_to_string(&Path) {
				Ok(Content) => {
					crate::WorkSpace::WorkSpaceFileService::ParseWorkSpaceFile(&Path, &Content)
						.map(|Folders| (Folders, Some(Path)))
						.unwrap_or_else(|Error| {
							error!(
								"[Main] Failed to parse workspace file: {}. Continuing without workspace.",
								Error
							);

							(Vec::new(), None)
						})
				},

				Err(Error) => {
					error!("[Main] Failed to read workspace file: {}. Continuing without workspace.", Error);

					(Vec::new(), None)
				},
			}
		} else {
			(Vec::new(), None)
		};

		// --- State Initialization ---
		let AppState = Arc::new(ApplicationState {
			WorkSpaceFolders:Arc::new(Mutex::new(InitialFolders)),

			WorkSpaceConfigurationPath:Arc::new(Mutex::new(WorkSpaceConfigurationPath)),

			..ApplicationState::default()
		});

		// --- Scheduler Initialization ---
		let NumberOfWorkers = num_cpus::get().max(2);

		let Scheduler = SchedulerBuilder::Create().WithWorkerCount(NumberOfWorkers).Build();

		let SchedulerForShutdown = Arc::new(Scheduler);

		let SchedulerForRunTime = SchedulerForShutdown.clone();

		// --- Tauri Application Builder ---
		#[allow(unused_mut)]
		let mut Builder = tauri::Builder::default();

		#[cfg(any(windows, target_os = "linux"))]
		{
			Builder = Builder.any_thread();
		}

		Builder
			.manage(AppState.clone())
			.setup(move |Application| {
				info!("[Setup] Tauri setup hook initiated.");

				let ApplicationHandle = Application.handle().clone();

				// Register all native (Rust) commands first.
				Command::Bootstrap::RegisterNativeCommands(&ApplicationHandle, &AppState)
					.expect("Failed to register native commands.");

				// --- Window Creation ---
				let mut WindowBuilder = tauri::WebviewWindowBuilder::new(
					Application,
					// Use "main" as the label for consistency
					"main",
					tauri::WebviewUrl::App("Application/index.html".into()),
				)
				.use_https_scheme(true)
				.zoom_hotkeys_enabled(true)
				.browser_extensions_enabled(false);

				#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
				{
					WindowBuilder = WindowBuilder.title("Mountain").maximized(true).decorations(false).shadow(true);
				}

				#[allow(unused_variables)]
				let MainWindow = WindowBuilder.build().expect("Main application window build failed");

				#[cfg(debug_assertions)]
				MainWindow.open_devtools();

				// --- Backend Initialization ---
				let Environment = Arc::new(MountainEnvironment::Create(ApplicationHandle.clone()));

				let RunTime = Arc::new(ApplicationRunTime::Create(SchedulerForRunTime, Environment.clone()));

				ApplicationHandle.manage(RunTime);

				info!("[Setup] Echo scheduler and ApplicationRunTime created and managed.");

				// --- Post-Setup Initialization Task ---
				let PostSetupApplicationHandle = ApplicationHandle.clone();

				let PostSetupEnvironment = Environment.clone();

				tauri::async_runtime::spawn(async move {
					info!("[SetupTask] Starting post-setup initializations...");

					let AppStateForSetup = PostSetupEnvironment.ApplicationState.clone();

					if let Err(Error) = InitializeAndMergeConfigurations(&PostSetupEnvironment).await {
						error!("[SetupTask] Failed to initialize configuration: {}", Error);
					}

					{
						let mut ScanPathsGuard = AppStateForSetup
							.ExtensionScanPaths
							.lock()
							.map_err(MapLockError)
							.expect("Failed to lock ExtensionScanPaths");

						if let Ok(ExecutableDirectory) = std::env::current_exe() {
							if let Some(Parent) = ExecutableDirectory.parent() {
								ScanPathsGuard.push(Parent.join("../Resources/extensions"));

								ScanPathsGuard.push(Parent.join("extensions"));
							}
						}

						info!("[SetupTask] Extension scan paths initialized: {:?}", *ScanPathsGuard);
					}

					if let Err(Error) =
						ScanAndPopulateExtensions(PostSetupApplicationHandle.clone(), &AppStateForSetup).await
					{
						error!("[SetupTask] Failed to scan for extensions: {}", Error);
					}

					if let Err(Error) = Vine::Server::Initialize::Initialize(
						PostSetupApplicationHandle.clone(),
						"[::1]:50051".to_string(),
					) {
						error!("[SetupTask] Failed to initialize Vine gRPC server: {}", Error);
					}

					if let Err(Error) = InitializeCocoon(&PostSetupApplicationHandle, &PostSetupEnvironment).await {
						error!("[SetupTask] Failed to initialize Cocoon: {}", Error);
					}

					info!("[SetupTask] Post-setup initializations complete.");
				});

				Ok(())
			})
			.plugin(tauri_plugin_dialog::init())
			.plugin(tauri_plugin_fs::init())
			.invoke_handler(tauri::generate_handler![
				MountainGetWorkbenchConfiguration,
				Command::TreeView::GetTreeViewChildren,
				Command::LanguageFeature::MountainProvideHover,
				Command::LanguageFeature::MountainProvideCompletions,
				Command::LanguageFeature::MountainProvideDefinition,
				Command::LanguageFeature::MountainProvideReferences,
				Command::SourceControlManagement::GetAllSourceControlManagementState,
				Command::Keybinding::GetResolvedKeybinding,
				crate::Track::DispatchLogic::DispatchFrontendCommand,
				crate::Track::DispatchLogic::ResolveUIRequest,
			])
			.build(tauri::generate_context!())
			.expect("FATAL: Error while building Mountain Tauri application")
			.run(move |ApplicationHandle, Event| {
				if let RunEvent::ExitRequested { api, .. } = Event {
					info!("[RunEvent] Exit requested. Initiating graceful shutdown...");

					api.prevent_exit();

					let SchedulerHandle = SchedulerForShutdown.clone();

					let ApplicationHandleClone = ApplicationHandle.clone();

					tokio::spawn(async move {
						if let Some(RunTime) = ApplicationHandleClone.try_state::<Arc<ApplicationRunTime>>() {
							RunTime.inner().clone().Shutdown().await;
						} else {
							error!("[Shutdown] Could not retrieve ApplicationRunTime to shut down services.");
						}

						info!("[Shutdown] Shutting down Echo scheduler...");

						if let Ok(mut Scheduler) = Arc::try_unwrap(SchedulerHandle) {
							Scheduler.Stop().await;
						} else {
							error!("[Shutdown] Could not get exclusive ownership of scheduler for shutdown.");
						}

						info!("[Shutdown] Shutdown complete. Exiting application.");

						ApplicationHandleClone.exit(0);
					});
				}
			});

		info!("[Main] Mountain application has shut down.");
	});
}
