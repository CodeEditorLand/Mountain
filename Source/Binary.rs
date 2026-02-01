//! # Binary
//!
//! Main entry point for the Mountain native host application.
//!
//! ## RESPONSIBILITIES
//!
//! ### Application Lifecycle Management
//! - Orchestrate the complete application lifecycle from startup to shutdown
//! - Initialize all core services in the correct dependency order
//! - Manage graceful shutdown with proper cleanup of all resources
//! - Handle system tray events and native OS integration
//! - Coordinate service dependencies and health checks
//!
//! ### Service Initialization
//! - Configure and initialize Tauri application framework
//! - Set up structured logging with appropriate level filtering
//! - Initialize Echo task scheduler with optimal worker configuration
//! - Create ApplicationState with workspace management
//! - Start ApplicationRunTime effect execution engine
//! - Launch Vine gRPC server for inter-service communication
//! - Spawn Cocoon sidecar process for build tool support
//! - Connect to Air provider for AI/integration features
//! - Initialize IPC server and status reporting services
//!
//! ### Resource Management
//! - Select and bind localhost port for Service Worker support
//! - Parse command-line arguments for workspace opening
//! - Register native commands for frontend communication
//! - Create and configure main application window
//! - Manage system tray icon and menu interactions
//! - Handle configuration merging and extension discovery
//!
//! ### Error Handling & Recovery
//! - Implement graceful degradation when services are unavailable
//! - Provide comprehensive error logging for diagnostics
//! - Retry operations with exponential backoff where appropriate
//! - Ensure no resource leaks during startup failures
//! - Maintain application stability under error conditions
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Entry point (binary) of the Mountain application
//! - Orchestrator for all Mountain subsystems
//! - Bridge between Tauri framework and custom Mountain services
//!
//! ### Dependencies
//! - Tauri: Desktop application framework
//! - Echo: Task scheduling and execution
//! - Common: Shared infrastructure (ApplicationRunTime trait, Environment)
//! - ApplicationRunTime: Effect execution engine
//! - MountainEnvironment: Capability provider
//! - Vine: gRPC inter-service communication
//! - Cocoon: Build tool sidecar
//! - Air: AI/integration services
//!
//! ### Dependents
//! - Frontend (Sky): Receives commands and configuration
//! - All Mountain services: Initialized and managed by this binary
//!
//! ### VSCode Patterns Borrowed
//! - Multi-phase initialization sequence (Electron main process)
//! - Service health checks with graceful degradation
//! - Configuration merging from multiple sources
//! - Extension scanning and discovery
//! - Lifecycle event handling (setup, ready, exit)
//!
//! ## TODO
//!
//! ### Immediate Improvements
//! - Add telemetry/analytics for startup performance
//! - Implement service dependency graph with automatic ordering
//! - Add startup timeout with diagnostic report
//! - Implement hot-reload for development builds
//!
//! ### Future Work
//! - Add crash reporter integration
//! - Implement service auto-restart on failure
//! - Add performance profiling and metrics collection
//! - Implement distributed logging for multi-instance scenarios
//! - Add plugin system for third-party services
//! - Implement secure IPC channel validation
//! - Add watchdog timer for stalled services
//!
//! ### Missing Functionality to Probe
//! - Service health check intervals and thresholds
//! - Optimal worker count calculation for Echo scheduler
//! - Port conflict resolution strategy
//! - Extension version compatibility checking
//! - Cocoon communication protocol details
//! - Air provider fallback strategy
//!
//! ## Logging Strategy
//!
//! - Release default: Info (low noise) unless RUST_LOG overrides
//! - Debug default: Debug (high fidelity) unless RUST_LOG overrides
//! - Very noisy deps are capped using level_for(...) and filter(...)
//!
//! ## Webview Logs
//!
//! To see Rust logs in the Webview console, enable TargetKind::Webview and
//! call attachConsole() in the frontend.

use std::{
	path::PathBuf,
	sync::{Arc, Mutex},
};

use Echo::Scheduler::SchedulerBuilder::SchedulerBuilder;
use log::{LevelFilter, debug, error, info, trace, warn};
use tauri::{
	AppHandle,
	Manager,
	RunEvent,
	Wry,
	image::Image,
	menu::{MenuBuilder, MenuItem},
	tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

use crate::{
	Air,
	ApplicationState::{
		ApplicationState::{ApplicationState, MapLockError},
		Internal::ScanAndPopulateExtensions,
	},
	Command,
	Environment::{ConfigurationProvider::InitializeAndMergeConfigurations, MountainEnvironment::MountainEnvironment},
	IPC::{
		TauriIPCServer::TauriIPCServer,
		initialize_advanced_features,
		initialize_status_reporter,
		initialize_wind_advanced_sync,
		register_wind_ipc_handlers,
	},
	ProcessManagement::{CocoonManagement::InitializeCocoon, InitializationData},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Vine,
};

// =============================================================================
// Debug Helpers (Highly Verbose, Low Intrusion)
// =============================================================================

/// Logs a checkpoint message at TRACE level (for "every step" tracing).
macro_rules! TraceStep {
	($($arg:tt)*) => {{
		trace!($($arg)*);
	}};
}

// =============================================================================
// IPC Bridge Commands
// =============================================================================

/// A Tauri command to provide the initial workbench configuration to the Sky
/// frontend.
#[tauri::command]
async fn MountainGetWorkbenchConfiguration(
	ApplicationHandle:AppHandle,
	State:tauri::State<'_, Arc<ApplicationState>>,
) -> Result<serde_json::Value, String> {
	info!("[IPC] [WorkbenchConfig] Request received.");

	debug!("[IPC] [WorkbenchConfig] Constructing sandbox configuration...");

	let Config = InitializationData::ConstructSandboxConfiguration(&ApplicationHandle, &State)
		.await
		.map_err(|Error| {
			error!("[IPC] [WorkbenchConfig] Failed: {}", Error);

			Error.to_string()
		})?;

	debug!("[IPC] [WorkbenchConfig] Success. Returning payload.");

	Ok(Config)
}

/// Dynamically switches the tray icon based on the theme (Light/Dark).
/// Can be invoked from the frontend when the theme changes.
#[tauri::command]
fn SwitchTrayIcon(App:AppHandle, IsDarkMode:bool) {
	debug!("[UI] [Tray] Switching icon. IsDarkMode: {}", IsDarkMode);

	const DARK_ICON_BYTES:&[u8] = include_bytes!("../icons/32x32.png");

	const LIGHT_ICON_BYTES:&[u8] = include_bytes!("../icons/32x32.png");

	let IconBytes = if IsDarkMode { DARK_ICON_BYTES } else { LIGHT_ICON_BYTES };

	if let Some(Tray) = App.tray_by_id("tray") {
		match Image::from_bytes(IconBytes) {
			Ok(IconImage) => {
				if let Err(e) = Tray.set_icon(Some(IconImage)) {
					error!("[UI] [Tray] Failed to set icon: {}", e);
				}
			},
			Err(e) => error!("[UI] [Tray] Failed to load icon bytes: {}", e),
		}
	} else {
		warn!("[UI] [Tray] Tray with ID 'tray' not found.");
	}
}

// =============================================================================
// IPC Command Wrappers
// =============================================================================

/// Receive messages from Wind through IPC
#[tauri::command]
async fn MountainIPCReceiveMessage(
	app_handle:AppHandle,
	message:serde_json::Value,
) -> Result<serde_json::Value, String> {
	crate::IPC::TauriIPCServer::mountain_ipc_receive_message(
		app_handle,
		serde_json::from_value(message).map_err(|e| e.to_string())?,
	)
	.await
}

/// Get Mountain IPC status
#[tauri::command]
async fn MountainIPCGetStatus(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	let status = crate::IPC::TauriIPCServer::mountain_ipc_get_status(app_handle)
		.await
		.map_err(|e| {
			error!("[IPC] [Command] Failed to get IPC status: {}", e);
			e.to_string()
		})?;
	Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
}

/// Invoke IPC methods
#[tauri::command]
async fn MountainIPCInvoke(
	app_handle:AppHandle,
	method:String,
	params:serde_json::Value,
) -> Result<serde_json::Value, String> {
	crate::IPC::WindServiceHandlers::mountain_ipc_invoke(app_handle, method, params).await
}

/// Get Wind desktop configuration
#[tauri::command]
async fn MountainGetWindDesktopConfiguration(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::mountain_get_wind_desktop_configuration(app_handle).await
}

/// Update configuration from Wind
#[tauri::command]
async fn MountainUpdateConfigurationFromWind(
	app_handle:AppHandle,
	config:serde_json::Value,
) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::mountain_update_configuration_from_wind(app_handle, config).await
}

/// Synchronize configuration
#[tauri::command]
async fn MountainSynchronizeConfiguration(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::mountain_synchronize_configuration(app_handle).await
}

/// Get configuration status
#[tauri::command]
async fn MountainGetConfigurationStatus(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::mountain_get_configuration_status(app_handle).await
}

/// Get configuration data for Wind frontend
#[tauri::command]
async fn get_configuration_data(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::get_configuration_data(app_handle).await
}

/// Save configuration data from Wind frontend
#[tauri::command]
async fn save_configuration_data(app_handle:AppHandle, config_data:serde_json::Value) -> Result<(), String> {
	crate::IPC::ConfigurationBridge::save_configuration_data(app_handle, config_data).await
}

/// Get IPC status
#[tauri::command]
async fn MountainGetIPCStatus(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::StatusReporter::mountain_get_ipc_status(app_handle).await
}

/// Get IPC status history
#[tauri::command]
async fn MountainGetIPCStatusHistory(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::StatusReporter::mountain_get_ipc_status_history(app_handle).await
}

/// Start IPC status reporting
#[tauri::command]
async fn MountainStartIPCStatusReporting(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::StatusReporter::mountain_start_ipc_status_reporting(app_handle, 60).await
}

/// Get performance stats
#[tauri::command]
async fn MountainGetPerformanceStats(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::AdvancedFeatures::mountain_get_performance_stats(app_handle).await
}

/// Get cache stats
#[tauri::command]
async fn MountainGetCacheStats(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::AdvancedFeatures::mountain_get_cache_stats(app_handle).await
}

/// Create collaboration session
#[tauri::command]
async fn MountainCreateCollaborationSession(
	app_handle:AppHandle,
	session_data:serde_json::Value,
) -> Result<serde_json::Value, String> {
	crate::IPC::AdvancedFeatures::mountain_create_collaboration_session(app_handle, session_data).await
}

/// Get collaboration sessions
#[tauri::command]
async fn MountainGetCollaborationSessions(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::AdvancedFeatures::mountain_get_collaboration_sessions(app_handle).await
}

/// Add document for sync
#[tauri::command]
async fn MountainAddDocumentForSync(
	app_handle:AppHandle,
	document_data:serde_json::Value,
) -> Result<serde_json::Value, String> {
	let document_id = document_data["document_id"]
		.as_str()
		.ok_or_else(|| {
			error!("[IPC] [Sync] Missing document_id in document_data");
			"Missing document_id"
		})?
		.to_string();
	let file_path = document_data["file_path"]
		.as_str()
		.ok_or_else(|| {
			error!("[IPC] [Sync] Missing file_path in document_data");
			"Missing file_path"
		})?
		.to_string();

	crate::IPC::WindAdvancedSync::mountain_add_document_for_sync(app_handle, document_id, file_path)
		.await
		.map_err(|e| {
			error!("[IPC] [Sync] Failed to add document for sync: {}", e);
			e.to_string()
		})
		.map(|_| serde_json::Value::Null)
}

/// Get sync status
#[tauri::command]
async fn MountainGetSyncStatus(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::WindAdvancedSync::mountain_get_sync_status(app_handle)
		.await
		.map_err(|e| {
			error!("[IPC] [Sync] Failed to get sync status: {}", e);
			e.to_string()
		})
		.map(|status| serde_json::to_value(status).unwrap_or(serde_json::Value::Null))
}

/// Subscribe to updates
#[tauri::command]
async fn MountainSubscribeToUpdates(
	app_handle:AppHandle,
	subscription_data:serde_json::Value,
) -> Result<serde_json::Value, String> {
	let target = subscription_data["target"]
		.as_str()
		.ok_or_else(|| {
			error!("[IPC] [Sync] Missing target in subscription_data");
			"Missing target"
		})?
		.to_string();
	let subscriber = subscription_data["subscriber"]
		.as_str()
		.ok_or_else(|| {
			error!("[IPC] [Sync] Missing subscriber in subscription_data");
			"Missing subscriber"
		})?
		.to_string();

	crate::IPC::WindAdvancedSync::mountain_subscribe_to_updates(app_handle, target, subscriber)
		.await
		.map_err(|e| {
			error!("[IPC] [Sync] Failed to subscribe to updates: {}", e);
			e.to_string()
		})
		.map(|_| serde_json::Value::Null)
}

// =============================================================================
// Tray Initialization Logic
// =============================================================================

/// Configures and builds the system tray with menu and event handling.
fn EnableTray(Application:&mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
	let Handle = Application.handle();

	// Create menu items
	let OpenItem = MenuItem::with_id(Handle, "open", "Open Mountain", true, None::<&str>)?;

	let HideItem = MenuItem::with_id(Handle, "hide", "Hide Mountain", true, None::<&str>)?;

	let Separator = tauri::menu::PredefinedMenuItem::separator(Handle)?;

	let QuitItem = MenuItem::with_id(Handle, "quit", "Quit", true, None::<&str>)?;

	// Build menu structure
	let TrayMenu = MenuBuilder::new(Handle)
		.item(&OpenItem)
		.item(&HideItem)
		.item(&Separator)
		.item(&QuitItem)
		.build()?;

	// Load initial icon (Defaulting to 32x32 from your icons folder)
	let IconBytes = include_bytes!("../icons/32x32.png");

	let TrayIconImage = Image::from_bytes(IconBytes)?;

	// Build the Tray
	TrayIconBuilder::with_id("tray")
		.icon(TrayIconImage)
		.menu(&TrayMenu)
		.tooltip("Mountain")
		// Handle Menu Item Clicks
		.on_menu_event(|AppHandle, Event| match Event.id.as_ref() {
			"open" => {
				if let Some(Window) = AppHandle.get_webview_window("main") {
					let _ = Window.show();

					let _ = Window.set_focus();

				}
			},
			"hide" => {
				if let Some(Window) = AppHandle.get_webview_window("main") {
					let _ = Window.hide();

				}
			},
			"quit" => AppHandle.exit(0),
			_ => warn!("[UI] [Tray] Unhandled menu item: {:?}", Event.id),
		})
		// Handle Native Tray Events (Left Click to Toggle)
		.on_tray_icon_event(|Tray, Event| {
			if let TrayIconEvent::Click { button: MouseButton::Left, .. } = Event {
				let App = Tray.app_handle();

				if let Some(Window) = App.get_webview_window("main") {
					if Window.is_visible().unwrap_or(false) {
						let _ = Window.hide();
					} else {
						let _ = Window.show();

						let _ = Window.set_focus();
					}
				}
			}
		})
		.build(Application)?;

	info!("[UI] [Tray] System tray enabled successfully.");

	Ok(())
}

// =============================================================================
// Binary Entrypoint
// =============================================================================

/// The main asynchronous function that sets up and runs the application.
pub fn Fn() {
	// -------------------------------------------------------------------------
	// [Boot] [Runtime] Tokio runtime creation
	// -------------------------------------------------------------------------
	TraceStep!("[Boot] [Runtime] Building Tokio runtime...");

	let Runtime = tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("FATAL: Cannot build Tokio runtime.");

	TraceStep!("[Boot] [Runtime] Tokio runtime built.");

	Runtime.block_on(async {
		// ---------------------------------------------------------------------
		// [Boot] [Args] CLI parsing
		// ---------------------------------------------------------------------
		debug!("[Boot] [Args] Collecting CLI args...");

		let CliArgs:Vec<String> = std::env::args().collect();

		debug!("[Boot] [Args] CLI Args: {:?}", CliArgs);

		let WorkSpacePathArgument = CliArgs.iter().find(|Arg| Arg.ends_with(".code-workspace"));

		TraceStep!("[Boot] [Args] Workspace arg present: {}", WorkSpacePathArgument.is_some());

		let (InitialFolders, WorkSpaceConfigurationPath) = if let Some(PathString) = WorkSpacePathArgument {
			let Path = PathBuf::from(PathString);

			println!("[Boot] [Args] Found workspace argument: {}", Path.display());

			debug!("[Boot] [Workspace] Reading workspace file: {}", Path.display());

			match std::fs::read_to_string(&Path) {
				Ok(Content) => {
					debug!("[Boot] [Workspace] Workspace file read ok ({} bytes).", Content.len());

					crate::WorkSpace::WorkSpaceFileService::ParseWorkSpaceFile(&Path, &Content)
						.map(|Folders| {
							debug!("[Boot] [Workspace] Parsed workspace ok. folder_count={}", Folders.len());

							(Folders, Some(Path))
						})
						.unwrap_or_else(|Error| {
							error!("[Boot] [Workspace] Parse failed: {}. Continuing without workspace.", Error);

							(Vec::new(), None)
						})
				},
				Err(Error) => {
					error!("[Boot] [Workspace] Read failed: {}. Continuing without workspace.", Error);

					(Vec::new(), None)
				},
			}
		} else {
			debug!("[Boot] [Workspace] No workspace provided. Starting empty.");

			(Vec::new(), None)
		};

		// ---------------------------------------------------------------------
		// [Boot] [State] ApplicationState
		// ---------------------------------------------------------------------
		debug!("[Boot] [State] Building ApplicationState...");

		let AppState = Arc::new(ApplicationState {
			WorkSpaceFolders:Arc::new(Mutex::new(InitialFolders)),
			WorkSpaceConfigurationPath:Arc::new(Mutex::new(WorkSpaceConfigurationPath)),
			..ApplicationState::default()
		});

		debug!(
			"[Boot] [State] ApplicationState created with {} workspace folders.",
			AppState.WorkSpaceFolders.lock().map(|f| f.len()).unwrap_or(0)
		);

		// TODO: EchoScheduler::new() needed here when Echo crate is available
		// let SchedulerForShutdown = Arc::new(Echo::Scheduler::SchedulerBuilder::new().build());
		let SchedulerForShutdown = Arc::new(()); // Placeholder for now

		let SchedulerForRunTime = SchedulerForShutdown.clone();

		TraceStep!("[Boot] [Echo] Scheduler handles prepared.");

		// ---------------------------------------------------------------------
		// [Boot] [Localhost] Port selection (for Service Workers & stable origin)
		// ---------------------------------------------------------------------
		debug!("[Boot] [Localhost] Selecting unused port...");

		let ServerPort =
			portpicker::pick_unused_port().expect("FATAL: Failed to find a free port for Localhost Server");

		debug!("[Boot] [Localhost] Selected port={}", ServerPort);

		let LocalhostUrl = format!("http://localhost:{}", ServerPort);

		println!("[Boot] [Localhost] Selected: {} ({})", ServerPort, LocalhostUrl);

		// ---------------------------------------------------------------------
		// [Boot] [Logging] Log level resolution
		// ---------------------------------------------------------------------
		// Supported by tauri_plugin_log:
		// - level(...) sets the max log level
		// - level_for(...) overrides max level per module
		// - filter(...) discards by metadata
		// - format(...) custom line formatting
		// - targets([...]) configures outputs (Stdout/Webview/File)
		// - timezone_strategy/rotation_strategy for file behavior
		let EnvLogLevel = std::env::var("RUST_LOG").ok().and_then(|s| s.parse::<LevelFilter>().ok());

		let DefaultLogLevel = if cfg!(debug_assertions) { LevelFilter::Debug } else { LevelFilter::Info };

		let LogLevel = EnvLogLevel.unwrap_or(DefaultLogLevel);

		// Log *very early* using stderr/stdout (logger not yet installed).
		// Once the plugin is installed, subsequent logs will go through it.
		eprintln!(
			"[Boot] [Logging] Resolved LogLevel={:?} (env={:?}, default={:?})",
			LogLevel, EnvLogLevel, DefaultLogLevel
		);

		// ---------------------------------------------------------------------
		// [Boot] [Tauri] Builder
		// ---------------------------------------------------------------------
		#[allow(unused_mut)]
		let mut Builder = tauri::Builder::default();

		#[cfg(any(windows, target_os = "linux"))]
		{
			Builder = Builder.any_thread();
		}

		Builder
			// -----------------------------------------------------------------
			// [Boot] [Tauri] [Plugins] Logging
			// -----------------------------------------------------------------
			.plugin(
				tauri_plugin_log::Builder::new()
					.targets([
						Target::new(TargetKind::Stdout),
						Target::new(TargetKind::LogDir {
							file_name: Some("Mountain.log".into()),
						}),
						Target::new(TargetKind::Webview),
					])
					.timezone_strategy(TimezoneStrategy::UseLocal)
					.rotation_strategy(RotationStrategy::KeepAll)
					.level(LogLevel)
					// Cap common noisy deps even when RUST_LOG=trace.
					.level_for("hyper", LevelFilter::Info)
					.level_for("mio", LevelFilter::Info)
					.level_for("tao", LevelFilter::Info)
					.level_for("tracing", LevelFilter::Info)
					// Drop noise by target metadata.
					.filter(|Metadata| {
						!Metadata.target().starts_with("polling")
							&& !Metadata.target().starts_with("tokio_reactor")
							&& !Metadata.target().starts_with("want")
					})
					// Add category-like formatting: DATE [LEVEL] [TARGET] message.
					.format(|out, message, record| {
						out.finish(format_args!(
							"[{:<5}] [{}] {}",
							record.level(),
							record.target(),
							message
						))
					})
					.build(),
			)
			// -----------------------------------------------------------------
			// [Boot] [Tauri] [Plugins] Localhost server (pre-selected port)
			// -----------------------------------------------------------------
			.plugin(tauri_plugin_localhost::Builder::new(ServerPort)
			.on_request(|_, Response| {
				Response.add_header("Access-Control-Allow-Origin", "*");

				Response.add_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS, HEAD");

				Response.add_header("Access-Control-Allow-Headers", "Content-Type, Authorization, Origin, Accept");
			})
			.build())
			// -----------------------------------------------------------------
			// [Boot] [DI] Global state
			// -----------------------------------------------------------------
			.manage(AppState.clone())
			// -----------------------------------------------------------------
			// [Lifecycle] Setup hook
			// -----------------------------------------------------------------
			.setup({
				let LocalhostUrl = LocalhostUrl.clone();

				move |Application| {
					info!("[Lifecycle] [Setup] Setup hook started.");

					debug!("[Lifecycle] [Setup] LocalhostUrl={}", LocalhostUrl);

					let ApplicationHandle = Application.handle().clone();

					TraceStep!("[Lifecycle] [Setup] AppHandle acquired.");

					// ---------------------------------------------------------
					// [UI] [Tray] Initialize System Tray
					// ---------------------------------------------------------
					debug!("[UI] [Tray] Initializing system tray...");

					if let Err(Error) = EnableTray(Application) {
						error!("[UI] [Tray] Failed to enable tray: {}", Error);

						// We do not crash the app if tray fails, but we log it.
					}

					// ---------------------------------------------------------
					// [Lifecycle] [Commands] Bootstrap native commands
					// ---------------------------------------------------------
					debug!("[Lifecycle] [Commands] Registering native commands...");

					Command::Bootstrap::RegisterNativeCommands(&ApplicationHandle, &AppState)
						.expect("FATAL: Failed to register native commands.");

					debug!("[Lifecycle] [Commands] Native commands registered.");

				// ---------------------------------------------------------
				// [Lifecycle] [IPC] Initialize Mountain IPC Server
				// ---------------------------------------------------------
				debug!("[Lifecycle] [IPC] Initializing Mountain IPC Server...");
	
				let ipc_server = TauriIPCServer::new(ApplicationHandle.clone());
				ApplicationHandle.manage(ipc_server.clone());
	
				debug!("[Lifecycle] [IPC] Mountain IPC Server initialized.");

					TraceStep!("[UI] [Window] InitScript bytes=0");

					debug!("[UI] [Window] Creating window builder...");

					let mut WindowBuilder = tauri::WebviewWindowBuilder::new(
						Application,
						"main",
						tauri::WebviewUrl::External(
							format!("{}/Application/index.html", LocalhostUrl).parse().unwrap(),
						),
					)
					.use_https_scheme(false)
					.initialization_script("")
					.zoom_hotkeys_enabled(true)
					.browser_extensions_enabled(false);

					#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
					{
						WindowBuilder = WindowBuilder
							.title("Mountain")
							.maximized(true)
							.decorations(false)
							.shadow(true);
					}

					debug!("[UI] [Window] Building main window...");

					#[allow(unused_variables)]
					let MainWindow = WindowBuilder.build().expect("FATAL: Main window build failed");

					info!("[UI] [Window] Main window ready.");

					#[cfg(debug_assertions)]
					{
						debug!("[UI] [Window] Debug build: opening DevTools.");

						MainWindow.open_devtools();
					}

					// ---------------------------------------------------------
					// [Backend] [Env] Mountain environment
					// ---------------------------------------------------------
					debug!("[Backend] [Env] Creating MountainEnvironment...");

					let Environment = Arc::new(MountainEnvironment::Create(ApplicationHandle.clone()));

					info!("[Backend] [Env] MountainEnvironment ready.");

					// ---------------------------------------------------------
					// [Backend] [Runtime] ApplicationRunTime
					// ---------------------------------------------------------
					debug!("[Backend] [Runtime] Creating ApplicationRunTime...");

					let RunTime = Arc::new(ApplicationRunTime::Create(
						SchedulerForRunTime.clone(),
						Environment.clone(),
					));

					ApplicationHandle.manage(RunTime.clone());

					info!("[Backend] [Runtime] ApplicationRunTime managed.");

					// ---------------------------------------------------------
					// [Lifecycle] [IPC] Initialize Status Reporter
					// ---------------------------------------------------------
					debug!("[Lifecycle] [IPC] Initializing Status Reporter...");

					if let Err(e) = initialize_status_reporter(&ApplicationHandle, RunTime.clone()) {
						error!("[Lifecycle] [IPC] Failed to initialize status reporter: {}", e);
					}

					debug!("[Lifecycle] [IPC] Status Reporter initialized.");

					// ---------------------------------------------------------
					// [Lifecycle] [IPC] Initialize Advanced Features
					// ---------------------------------------------------------
					debug!("[Lifecycle] [IPC] Initializing Advanced Features...");

					if let Err(e) = initialize_advanced_features(&ApplicationHandle, RunTime.clone()) {
						error!("[Lifecycle] [IPC] Failed to initialize advanced features: {}", e);
					}

					debug!("[Lifecycle] [IPC] Advanced Features initialized.");

					// ---------------------------------------------------------
					// [Lifecycle] [IPC] Initialize Wind Advanced Sync
					// ---------------------------------------------------------
					debug!("[Lifecycle] [IPC] Initializing Wind Advanced Sync...");

					if let Err(e) = initialize_wind_advanced_sync(&ApplicationHandle, RunTime.clone()) {
						error!("[Lifecycle] [IPC] Failed to initialize Wind advanced sync: {}", e);
					}

					debug!("[Lifecycle] [IPC] Wind Advanced Sync initialized.");

					// ---------------------------------------------------------
					// [Air] [gRPC] Initialize Air client
					// ---------------------------------------------------------
					// TODO: Air integration not fully implemented - commenting out Air client initialization
					/*
					debug!("[Air] [Init] Initializing Air client...");

					let AirAddress = "http://[::1]:50053";

					// Attempt to connect to Air, but continue gracefully if unavailable
					let AirProvider = match Air::AirServiceProvider::new(&AirAddress) {
						Ok(provider) => {
							info!("[Air] [Init] Successfully connected to Air at {}", AirAddress);
							provider
						},
						Err(e) => {
							warn!("[Air] [Init] Failed to connect to Air: {}. Continuing without Air client.", e);
							// Create unavailable provider
							Air::AirServiceProvider::new_unavailable()
						},
					};

					// Store AirServiceProvider in Tauri state for global access
					ApplicationHandle.manage(AirProvider);

					// Log Air availability status
					if let Some(air_provider) = ApplicationHandle.try_state::<Air::AirServiceProvider>() {
						if air_provider.is_available() {
							info!("[Air] [Init] Air is available and connected.");
						} else {
							warn!("[Air] [Init] Air is not connected. Some features may be unavailable.");
						}
					}
					*/
					debug!("[Air] [Init] Air integration skipped (not fully implemented).");

					// ---------------------------------------------------------
					// [Lifecycle] [PostSetup] Async initialization work
					// ---------------------------------------------------------
					let PostSetupApplicationHandle = ApplicationHandle.clone();

					let PostSetupEnvironment = Environment.clone();

					tauri::async_runtime::spawn(async move {
						info!("[Lifecycle] [PostSetup] Starting...");

						let AppStateForSetup = PostSetupEnvironment.ApplicationState.clone();

						TraceStep!("[Lifecycle] [PostSetup] AppState cloned.");

						// [Config]
						debug!("[Config] InitializeAndMergeConfigurations starting...");

						if let Err(Error) = InitializeAndMergeConfigurations(&PostSetupEnvironment).await {
							error!("[Config] InitializeAndMergeConfigurations failed: {}", Error);
						} else {
							info!("[Config] InitializeAndMergeConfigurations done.");
						}

						// [Extensions] [ScanPaths]
						{
							debug!("[Extensions] [ScanPaths] Locking ExtensionScanPaths...");

							let mut ScanPathsGuard = AppStateForSetup
								.ExtensionScanPaths
								.lock()
								.map_err(MapLockError)
								.expect("FATAL: Failed to lock ExtensionScanPaths");

							debug!("[Extensions] [ScanPaths] Adding default scan paths...");

							if let Ok(ExecutableDirectory) = std::env::current_exe() {
								if let Some(Parent) = ExecutableDirectory.parent() {
									let ResourcesPath = Parent.join("../Resources/extensions");

									let LocalPath = Parent.join("extensions");

									debug!(
										"[Extensions] [ScanPaths] + {}",
										ResourcesPath.display()
									);

									ScanPathsGuard.push(ResourcesPath);

									debug!(
										"[Extensions] [ScanPaths] + {}",
										LocalPath.display()
									);

									ScanPathsGuard.push(LocalPath);

								}
							}

							info!(
								"[Extensions] [ScanPaths] Initialized: {:?}",
								*ScanPathsGuard
							);
						}

						// [Extensions] [Scan]
						debug!("[Extensions] [Scan] ScanAndPopulateExtensions starting...");

						if let Err(Error) =
							ScanAndPopulateExtensions(PostSetupApplicationHandle.clone(), &AppStateForSetup).await
						{
							error!("[Extensions] [Scan] Failed: {}", Error);
						} else {
							info!("[Extensions] [Scan] Completed.");
						}

						// [Vine] [gRPC]
						debug!("[Vine] [Init] Starting Vine gRPC server...");

						if let Err(Error) = Vine::Server::Initialize::Initialize(
							PostSetupApplicationHandle.clone(),
							"[::1]:50051".to_string(),						"[::1]:50052".to_string(),						) {
							error!("[Vine] [Init] Failed: {}", Error);
						} else {
							info!("[Vine] [Init] Ready.");
						}

						// [Cocoon] [Sidecar]
						debug!("[Cocoon] [Init] InitializeCocoon starting...");

						if let Err(Error) = InitializeCocoon(&PostSetupApplicationHandle, &PostSetupEnvironment).await {
							error!("[Cocoon] [Init] Failed: {}", Error);
						} else {
							info!("[Cocoon] [Init] Ready.");
						}

						info!("[Lifecycle] [PostSetup] Complete. System ready.");
					});

					Ok(())
				}
			})
			// ---------------------------------------------------------------------
			// [Tauri] [Plugins] Standard
			// ---------------------------------------------------------------------
			.plugin(tauri_plugin_dialog::init())
			.plugin(tauri_plugin_fs::init())
			// ---------------------------------------------------------------------
			// [IPC] Command routing
			// ---------------------------------------------------------------------
			.invoke_handler(tauri::generate_handler![
				SwitchTrayIcon,
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
				MountainIPCReceiveMessage,
				MountainIPCGetStatus,
				MountainIPCInvoke,
				MountainGetWindDesktopConfiguration,
				MountainUpdateConfigurationFromWind,
				MountainSynchronizeConfiguration,
				MountainGetConfigurationStatus,
				MountainGetIPCStatus,
				MountainGetIPCStatusHistory,
				MountainStartIPCStatusReporting,
				MountainGetPerformanceStats,
				MountainGetCacheStats,
				MountainCreateCollaborationSession,
				MountainGetCollaborationSessions,
				MountainAddDocumentForSync,
				MountainGetSyncStatus,
				MountainSubscribeToUpdates,
				get_configuration_data,
				save_configuration_data,
			])
			// ---------------------------------------------------------------------
			// [Tauri] Build & run loop
			// ---------------------------------------------------------------------
			.build(tauri::generate_context!())
			.expect("FATAL: Error while building Mountain Tauri application")
			.run(move |ApplicationHandle, Event| {
				// Debug-only: log selected lifecycle events (but avoid super-noisy ones).
				if cfg!(debug_assertions) {
					match &Event {
						RunEvent::MainEventsCleared => {},
						RunEvent::WindowEvent { .. } => {},
						_ => debug!("[Lifecycle] [RunEvent] {:?}", Event),
					}
				}

				if let RunEvent::ExitRequested { api, .. } = Event {
					warn!("[Lifecycle] [Shutdown] Exit requested. Starting graceful shutdown...");

					api.prevent_exit();

					let SchedulerHandle = SchedulerForShutdown.clone();

					let ApplicationHandleClone = ApplicationHandle.clone();

					tokio::spawn(async move {
						debug!("[Lifecycle] [Shutdown] Shutting down ApplicationRunTime...");

						if let Some(RunTime) = ApplicationHandleClone.try_state::<Arc<ApplicationRunTime>>() {
							RunTime.inner().clone().Shutdown().await;

							info!("[Lifecycle] [Shutdown] ApplicationRunTime stopped.");
						} else {
							error!("[Lifecycle] [Shutdown] ApplicationRunTime not found.");
						}

						debug!("[Lifecycle] [Shutdown] Stopping Echo scheduler...");

						if let Ok(mut Scheduler) = Arc::try_unwrap(SchedulerHandle) {
							Scheduler.Stop().await;

							info!("[Lifecycle] [Shutdown] Echo scheduler stopped.");
						} else {
							error!("[Lifecycle] [Shutdown] Scheduler not exclusively owned; cannot stop cleanly.");
						}

						info!("[Lifecycle] [Shutdown] Done. Exiting process.");

						ApplicationHandleClone.exit(0);
					});

				}
			});

		info!("[Lifecycle] [Exit] Mountain application has shut down.");
	});
}
