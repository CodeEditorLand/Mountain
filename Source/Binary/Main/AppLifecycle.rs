//! # AppLifecycle (Binary/Main)
//!
//! ## RESPONSIBILITIES
//!
//! Application lifecycle management for the Tauri application setup and
//! initialization. This module handles the complete setup process during the
//! Tauri setup hook, including tray initialization, command registration, IPC
//! server setup, window creation, environment configuration, and async service
//! initialization.
//!
//! ## ARCHITECTURAL ROLE
//!
//! The AppLifecycle module is the **initialization layer** in Mountain's
//! architecture:
//!
//! ```text
//! Tauri Builder Setup ──► AppLifecycle::AppLifecycleSetup()
//!                              │
//!                              ├─► Tray Initialization
//!                              ├─► Command Registration
//!                              ├─► IPC Server Setup
//!                              ├─► Window Building
//!                              ├─► Environment Setup
//!                              ├─► Runtime Setup
//!                              └─► Async Service Initialization
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **AppLifecycleSetup()**: Main setup function orchestrating all
//!   initialization
//! - **Tray Initialization**: System tray icon with Dark/Light mode support
//! - **Command Registration**: Native command registration with application
//!   state
//! - **IPC Server**: Mountain IPC server for frontend-backend communication
//! - **Window Building**: Main application window configuration
//! - **MountainEnvironment**: Environment context for application services
//! - **ApplicationRunTime**: Runtime context with scheduler and environment
//! - **Status Reporter**: IPC status reporting initialization
//! - **Advanced Features**: Advanced IPC features initialization
//! - **Wind Sync**: Wind advanced sync initialization
//! - **Async Initialization**: Post-setup async service initialization
//!
//! ## ERROR HANDLING
//!
//! Returns `Result<(), Box<dyn std::error::Error>>` for setup errors.
//! Non-critical failures are logged but don't prevent application startup.
//! Critical failures are propagated to prevent incomplete startup.
//!
//! ## LOGGING
//!
//! Comprehensive logging at INFO level for major setup steps,
//! DEBUG level for detailed processing, and ERROR for failures.
//! All logs are prefixed with `[Lifecycle] [ComponentName]`.
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Async initialization spawned after main setup to avoid blocking
//! - Services initialized only when needed
//! - Clone operations minimized for Arc-wrapped shared state
//!
//! ## TODO
//! - [ ] Add setup progress tracking
//! - [ ] Implement setup timeout handling
//! - [ ] Add setup rollback mechanism on failure

use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use tauri::Manager;
use Echo::Scheduler::Scheduler::Scheduler;

use crate::{
	// Crate root imports
	ApplicationState::ApplicationState,
	// Binary submodule imports
	Binary::Build::WindowBuild::WindowBuild as WindowBuildFn,
	Binary::Extension::ExtensionPopulate::ExtensionPopulate as ExtensionPopulateFn,
	Binary::Extension::ScanPathConfigure::ScanPathConfigure as ScanPathConfigureFn,
	Binary::Register::AdvancedFeaturesRegister::AdvancedFeaturesRegister as AdvancedFeaturesRegisterFn,
	Binary::Register::CommandRegister::CommandRegister as CommandRegisterFn,
	Binary::Register::IPCServerRegister::IPCServerRegister as IPCServerRegisterFn,
	Binary::Register::StatusReporterRegister::StatusReporterRegister as StatusReporterRegisterFn,
	Binary::Register::WindSyncRegister::WindSyncRegister as WindSyncRegisterFn,
	Binary::Service::CocoonStart::CocoonStart as CocoonStartFn,
	Binary::Service::ConfigurationInitialize::ConfigurationInitialize as ConfigurationInitializeFn,
	Binary::Service::VineStart::VineStart as VineStartFn,
	Binary::Tray::EnableTray as EnableTrayFn,
	Environment::MountainEnvironment::MountainEnvironment,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Logs a checkpoint message at TRACE level.
macro_rules! TraceStep {
	($($arg:tt)*) => {{
		trace!($($arg)*);
	}};
}

/// Sets up the application lifecycle during Tauri initialization.
///
/// This function coordinates all setup operations:
/// 1. System tray initialization
/// 2. Native command registration
/// 3. IPC server initialization
/// 4. Main window creation
/// 5. Mountain environment setup
/// 6. Application runtime setup
/// 7. Status reporter initialization
/// 8. Advanced features initialization
/// 9. Wind advanced sync initialization
/// 10. Async post-setup initialization
///
/// # Parameters
///
/// * `app` - Mutable reference to Tauri App instance
/// * `app_handle` - Cloned Tauri AppHandle for async operations
/// * `localhost_url` - URL for the development server
/// * `scheduler` - Arc-wrapped Echo Scheduler
/// * `app_state` - Application state clone
///
/// # Returns
///
/// `Result<(), Box<dyn std::error::Error>>` - Ok on success, Err on critical
/// failure
pub fn AppLifecycleSetup(
	app:&mut tauri::App,
	app_handle:tauri::AppHandle,
	localhost_url:String,
	scheduler:Arc<Scheduler>,
	app_state:Arc<ApplicationState>,
) -> Result<(), Box<dyn std::error::Error>> {
	info!("[Lifecycle] [Setup] Setup hook started.");
	debug!("[Lifecycle] [Setup] LocalhostUrl={}", localhost_url);

	let app_handle_for_setup = app_handle.clone();
	TraceStep!("[Lifecycle] [Setup] AppHandle acquired.");

	// -------------------------------------------------------------------------
	// [UI] [Tray] Initialize System Tray
	// -------------------------------------------------------------------------
	debug!("[UI] [Tray] Initializing system tray...");
	if let Err(Error) = EnableTrayFn::enable_tray(app) {
		error!("[UI] [Tray] Failed to enable tray: {}", Error);
	}

	// -------------------------------------------------------------------------
	// [Lifecycle] [Commands] Register native commands
	// -------------------------------------------------------------------------
	debug!("[Lifecycle] [Commands] Registering native commands...");
	if let Err(e) = CommandRegisterFn(&app_handle_for_setup, &app_state) {
		error!("[Lifecycle] [Commands] Failed to register commands: {}", e);
	}
	debug!("[Lifecycle] [Commands] Native commands registered.");

	// -------------------------------------------------------------------------
	// [Lifecycle] [IPC] Initialize IPC Server
	// -------------------------------------------------------------------------
	debug!("[Lifecycle] [IPC] Initializing Mountain IPC Server...");
	if let Err(e) = IPCServerRegisterFn(&app_handle_for_setup) {
		error!("[Lifecycle] [IPC] Failed to register IPC server: {}", e);
	}

	// -------------------------------------------------------------------------
	// [UI] [Window] Build main window
	// -------------------------------------------------------------------------
	debug!("[UI] [Window] Building main window...");
	let MainWindow = WindowBuildFn(app, localhost_url.clone());
	info!("[UI] [Window] Main window ready.");

	#[cfg(debug_assertions)]
	{
		debug!("[UI] [Window] Debug build: opening DevTools.");
		MainWindow.open_devtools();
	}

	// -------------------------------------------------------------------------
	// [Backend] [Dirs] Ensure userdata directories exist
	// -------------------------------------------------------------------------
	{
		let PathResolver = app.path();
		let AppDataDir = PathResolver.app_data_dir().unwrap_or_default();
		let LogDir = PathResolver.app_log_dir().unwrap_or_default();
		let HomeDir = PathResolver.home_dir().unwrap_or_default();

		// Set the canonical userdata base so WindServiceHandlers resolves
		// /User/... paths to the real Tauri app_data_dir (not hardcoded "Land").
		crate::IPC::WindServiceHandlers::set_userdata_base_dir(AppDataDir.to_string_lossy().to_string());

		// Set the real filesystem root for /Static/Application/ path mapping.
		// In dev mode, Tauri serves from ../Sky/Target relative to Mountain.
		// Tauri's resource_dir gives us the frontendDist path.
		let SkyTargetDir = PathResolver.resource_dir().unwrap_or_else(|_| {
			// Fallback: resolve relative to the executable
			std::env::current_exe()
				.ok()
				.and_then(|Exe| Exe.parent().map(|P| P.to_path_buf()))
				.unwrap_or_default()
				.join("../../../Element/Sky/Target")
		});
		crate::IPC::WindServiceHandlers::set_static_application_root(SkyTargetDir.to_string_lossy().to_string());
		debug!("[Lifecycle] [Dirs] Static application root: {}", SkyTargetDir.display());

		// Every directory VS Code may stat or readdir during startup
		let Dirs = [
			// User profile directories
			AppDataDir.join("User"),
			AppDataDir.join("User/globalStorage"),
			AppDataDir.join("User/workspaceStorage"),
			AppDataDir.join("User/workspaceStorage/vscode-chat-images"),
			AppDataDir.join("User/extensions"),
			AppDataDir.join("User/profiles/__default__profile__"),
			AppDataDir.join("User/snippets"),
			AppDataDir.join("User/prompts"),
			AppDataDir.join("User/caches"),
			// Configuration cache
			AppDataDir.join("CachedConfigurations/defaults/__default__profile__-configurationDefaultsOverrides"),
			// Log directories - VS Code stats {logsPath}/window1/output_{timestamp}
			LogDir.join("window1"),
			// System extensions directory - VS Code scans appRoot/../extensions
			// which resolves to /Static/Application/extensions (mapped to Sky Target).
			SkyTargetDir.join("Static/Application/extensions"),
			// Agent directories VS Code probes for (create to avoid stat errors)
			HomeDir.join(".claude/agents"),
			HomeDir.join(".copilot/agents"),
		];
		for Dir in &Dirs {
			if let Err(Error) = std::fs::create_dir_all(Dir) {
				warn!("[Lifecycle] [Dirs] Failed to create {}: {}", Dir.display(), Error);
			}
		}

		// Default empty files VS Code reads on startup
		let DefaultFiles:&[(&std::path::Path, &str)] = &[
			(&AppDataDir.join("User/settings.json"), "{}"),
			(&AppDataDir.join("User/keybindings.json"), "[]"),
			(&AppDataDir.join("User/tasks.json"), "{}"),
			(&AppDataDir.join("User/extensions.json"), "[]"),
			(&AppDataDir.join("User/mcp.json"), "{}"),
		];
		for (FilePath, DefaultContent) in DefaultFiles {
			if !FilePath.exists() {
				let _ = std::fs::write(FilePath, DefaultContent);
			}
		}

		// Set GlobalMementoPath now that we know the real Tauri app data dir
		if let Ok(mut Path) = app_state.GlobalMementoPath.lock() {
			*Path = AppDataDir.join("User/globalStorage/global.json");
			debug!("[Lifecycle] [Dirs] GlobalMementoPath: {}", Path.display());
		}
		debug!("[Lifecycle] [Dirs] Userdata directories ensured at {}", AppDataDir.display());
	}

	// -------------------------------------------------------------------------
	// [Backend] [Env] Mountain environment
	// -------------------------------------------------------------------------
	debug!("[Backend] [Env] Creating MountainEnvironment...");
	let Environment = Arc::new(MountainEnvironment::Create(app_handle_for_setup.clone(), app_state.clone()));
	info!("[Backend] [Env] MountainEnvironment ready.");

	// -------------------------------------------------------------------------
	// [Backend] [Runtime] ApplicationRunTime
	// -------------------------------------------------------------------------
	debug!("[Backend] [Runtime] Creating ApplicationRunTime...");
	let Runtime = Arc::new(ApplicationRunTime::Create(scheduler.clone(), Environment.clone()));
	app_handle_for_setup.manage(Runtime.clone());
	info!("[Backend] [Runtime] ApplicationRunTime managed.");

	// -------------------------------------------------------------------------
	// [Lifecycle] [IPC] Initialize Status Reporter
	// -------------------------------------------------------------------------
	if let Err(e) = StatusReporterRegisterFn(&app_handle_for_setup, Runtime.clone()) {
		error!("[Lifecycle] [IPC] Failed to initialize status reporter: {}", e);
	}

	// -------------------------------------------------------------------------
	// [Lifecycle] [IPC] Initialize Advanced Features
	// -------------------------------------------------------------------------
	if let Err(e) = AdvancedFeaturesRegisterFn(&app_handle_for_setup, Runtime.clone()) {
		error!("[Lifecycle] [IPC] Failed to initialize advanced features: {}", e);
	}

	// -------------------------------------------------------------------------
	// [Lifecycle] [IPC] Initialize Wind Advanced Sync
	// -------------------------------------------------------------------------
	if let Err(e) = WindSyncRegisterFn(&app_handle_for_setup, Runtime.clone()) {
		error!("[Lifecycle] [IPC] Failed to initialize wind advanced sync: {}", e);
	}

	// -------------------------------------------------------------------------
	// [Lifecycle] [PostSetup] Async initialization work
	// -------------------------------------------------------------------------
	let PostSetupAppHandle = app_handle_for_setup.clone();
	let PostSetupEnvironment = Environment.clone();

	tauri::async_runtime::spawn(async move {
		info!("[Lifecycle] [PostSetup] Starting...");
		let AppStateForSetup = PostSetupEnvironment.ApplicationState.clone();
		TraceStep!("[Lifecycle] [PostSetup] AppState cloned.");

		// [Config]
		let _ = ConfigurationInitializeFn(&PostSetupEnvironment).await;

		// [Extensions] [ScanPaths]
		let _ = ScanPathConfigureFn(&AppStateForSetup);

		// [Extensions] [Scan]
		let _ = ExtensionPopulateFn(PostSetupAppHandle.clone(), &AppStateForSetup).await;

		// [Vine] [gRPC]
		let _ = VineStartFn(PostSetupAppHandle.clone(), "[::1]:50051".to_string(), "[::1]:50052".to_string()).await;

		// [Cocoon] [Sidecar]
		let _ = CocoonStartFn(&PostSetupAppHandle, &PostSetupEnvironment).await;

		info!("[Lifecycle] [PostSetup] Complete. System ready.");
	});

	Ok(())
}
