//! # Binary
//!
//! Main entry point for the Mountain native host application.
//!
//! Orchestration-focused file that coordinates application lifecycle from
//! startup to shutdown using extracted modules.

use std::{path::PathBuf, sync::Arc, time::Duration};

use log::{debug, error, info, trace, warn};
use serde_json::Value;
use tauri::{
	App,
	AppHandle,
	Manager,
	RunEvent,
	Wry,
	image::Image,
	menu::{MenuBuilder, MenuItem},
	tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use Echo::Scheduler::{Scheduler::Scheduler, SchedulerBuilder::SchedulerBuilder};
use CommonLibrary::Error::CommonError::CommonError;
use crate::ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO;

use crate::{
	// Binary submodule imports
	Binary::Build::LocalhostPlugin::LocalhostPlugin as LocalhostPluginFn,
	Binary::Build::LoggingPlugin::LoggingPlugin as LoggingPluginFn,
	Binary::Build::TauriBuild::TauriBuild as TauriBuildFn,
	Binary::Build::WindowBuild::WindowBuild as WindowBuildFn,
	Binary::Extension::ExtensionPopulate::ExtensionPopulate as ExtensionPopulateFn,
	Binary::Extension::ScanPathConfigure::ScanPathConfigure as ScanPathConfigureFn,
	Binary::Initialize::CliParse::Parse as CliParseFn,
	Binary::Initialize::LogLevel::Resolve as ResolveLogLevel,
	Binary::Initialize::PortSelector::BuildUrl as BuildPortUrl,
	Binary::Initialize::PortSelector::Select as SelectPort,
	Binary::Initialize::StateBuild::Build as BuildStateFn,
	Binary::Register::AdvancedFeaturesRegister::AdvancedFeaturesRegister as AdvancedFeaturesRegisterFn,
	Binary::Register::CommandRegister::CommandRegister as CommandRegisterFn,
	Binary::Register::IPCServerRegister::IPCServerRegister as IPCServerRegisterFn,
	Binary::Register::StatusReporterRegister::StatusReporterRegister as StatusReporterRegisterFn,
	Binary::Register::WindSyncRegister::WindSyncRegister as WindSyncRegisterFn,
	Binary::Service::CocoonStart::CocoonStart as CocoonStartFn,
	Binary::Service::ConfigurationInitialize::ConfigurationInitialize as ConfigurationInitializeFn,
	Binary::Service::VineStart::VineStart as VineStartFn,
	Binary::Shutdown::RuntimeShutdown::RuntimeShutdown as RuntimeShutdownFn,
	Binary::Shutdown::SchedulerShutdown::SchedulerShutdown as SchedulerShutdownFn,
	Binary::Tray::EnableTray as EnableTrayFn,
	// Crate root imports
	ApplicationState::ApplicationState::ApplicationState,
	Command,
	Environment::MountainEnvironment::MountainEnvironment,
	IPC::{
		AdvancedFeatures::CollaborationPermissions,
		TauriIPCServer::TauriIPCServer,
		initialize_wind_advanced_sync,
	},
	ProcessManagement::InitializationData,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::DispatchLogic,
};

/// Logs a checkpoint message at TRACE level.
macro_rules! TraceStep {
	($($arg:tt)*) => {{
		trace!($($arg)*);
	}};
}

// =============================================================================
// IPC Bridge Commands
// =============================================================================

#[tauri::command]
async fn MountainGetWorkbenchConfiguration(
	ApplicationHandle:AppHandle,
	State:tauri::State<'_, Arc<ApplicationState>>,
) -> Result<serde_json::Value, String> {
	info!("[IPC] [WorkbenchConfig] Request received.");

	debug!("[IPC] [WorkbenchConfig] Constructing sandbox configuration...");

	let Configuration = InitializationData::ConstructSandboxConfiguration(&ApplicationHandle, &State)
		.await
		.map_err(|Error: CommonError| {
			error!("[IPC] [WorkbenchConfig] Failed: {}", Error);
			Error.to_string()
		})?;

	debug!("[IPC] [WorkbenchConfig] Success. Returning payload.");

	Ok(Configuration)
}

#[tauri::command]
fn SwitchTrayIcon(App:AppHandle, IsDarkMode:bool) {
	debug!("[UI] [Tray] Switching icon. IsDarkMode: {}", IsDarkMode);

	const DARK_ICON_BYTES:&[u8] = include_bytes!("../../icons/32x32.png");
	const LIGHT_ICON_BYTES:&[u8] = include_bytes!("../../icons/32x32.png");

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
	.map(|()| serde_json::Value::Object(serde_json::Map::new()))
}

#[tauri::command]
async fn MountainIPCGetStatus(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	let status = crate::IPC::TauriIPCServer::mountain_ipc_get_status(app_handle)
		.await
		.map_err(|e: String| {
			error!("[IPC] [Command] Failed to get IPC status: {}", e);
			e.to_string()
		})?;
	Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn MountainIPCInvoke(
	app_handle:AppHandle,
	method:String,
	params:serde_json::Value,
) -> Result<serde_json::Value, String> {
	let args: Vec<Value> = params.as_array()
		.ok_or_else(|| "params must be an array".to_string())?
		.clone()
		.into_iter()
		.collect();
	
	crate::IPC::WindServiceHandlers::mountain_ipc_invoke(app_handle, method, args).await
}

#[tauri::command]
async fn MountainGetWindDesktopConfiguration(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::mountain_get_wind_desktop_configuration(app_handle)
		.await
		.and_then(|config| serde_json::to_value(config).map_err(|e: serde_json::Error| e.to_string()))
}

#[tauri::command]
async fn MountainUpdateConfigurationFromWind(
	app_handle:AppHandle,
	config:serde_json::Value,
) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::mountain_update_configuration_from_wind(app_handle, config)
		.await
		.map(|()| serde_json::json!({"success": true}))
}

#[tauri::command]
async fn MountainSynchronizeConfiguration(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::mountain_synchronize_configuration(app_handle)
		.await
		.map(|_| serde_json::json!({"synchronized": true}))
}

#[tauri::command]
async fn MountainGetConfigurationStatus(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::mountain_get_configuration_status(app_handle)
		.await
		.and_then(|status| serde_json::to_value(status).map_err(|e: serde_json::Error| e.to_string()))
}

#[tauri::command]
async fn get_configuration_data(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::ConfigurationBridge::get_configuration_data(app_handle)
		.await
		.and_then(|data| serde_json::to_value(data).map_err(|e: serde_json::Error| e.to_string()))
}

#[tauri::command]
async fn save_configuration_data(app_handle:AppHandle, config_data:serde_json::Value) -> Result<(), String> {
	crate::IPC::ConfigurationBridge::save_configuration_data(app_handle, config_data).await
}

#[tauri::command]
async fn MountainGetIPCStatus(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::StatusReporter::mountain_get_ipc_status(app_handle).await
}

#[tauri::command]
async fn MountainGetIPCStatusHistory(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::StatusReporter::mountain_get_ipc_status_history(app_handle).await
}

#[tauri::command]
async fn MountainStartIPCStatusReporting(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::StatusReporter::mountain_start_ipc_status_reporting(app_handle, 60).await
}

#[tauri::command]
async fn MountainGetPerformanceStats(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::AdvancedFeatures::mountain_get_performance_stats(app_handle)
		.await
		.and_then(|stats| serde_json::to_value(stats).map_err(|e: serde_json::Error| e.to_string()))
}

#[tauri::command]
async fn MountainGetCacheStats(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::AdvancedFeatures::mountain_get_cache_stats(app_handle)
		.await
		.and_then(|cache| serde_json::to_value(cache).map_err(|e: serde_json::Error| e.to_string()))
}

#[tauri::command]
async fn MountainCreateCollaborationSession(
	app_handle:AppHandle,
	session_data:serde_json::Value,
) -> Result<serde_json::Value, String> {
	let session_id = session_data["session_id"]
		.as_str()
		.ok_or_else(|| "Missing or invalid session_id in session_data".to_string())?
		.to_string();
	
	let permissions_list: Vec<String> = serde_json::from_value(session_data["permissions"].clone())
		.map_err(|e| format!("Invalid permissions: {}", e))?;
	
	// Convert Vec<String> to CollaborationPermissions
	let permissions = CollaborationPermissions {
		can_edit: permissions_list.iter().any(|p| p == "can_edit" || p == "edit"),
		can_view: permissions_list.iter().any(|p| p == "can_view" || p == "view"),
		can_comment: permissions_list.iter().any(|p| p == "can_comment" || p == "comment"),
		can_share: permissions_list.iter().any(|p| p == "can_share" || p == "share"),
	};
	
	let session_id_for_response = session_id.clone();
	crate::IPC::AdvancedFeatures::mountain_create_collaboration_session(app_handle, session_id, permissions)
		.await
		.map(|()| serde_json::json!({"session_id": session_id_for_response, "permissions": session_data["permissions"]}))
}

#[tauri::command]
async fn MountainGetCollaborationSessions(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::AdvancedFeatures::mountain_get_collaboration_sessions(app_handle)
		.await
		.and_then(|sessions| serde_json::to_value(sessions).map_err(|e: serde_json::Error| e.to_string()))
}

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
		.map_err(|e: String| {
			error!("[IPC] [Sync] Failed to add document for sync: {}", e);
			e.to_string()
		})
		.map(|_| serde_json::Value::Null)
}

#[tauri::command]
async fn MountainGetSyncStatus(app_handle:AppHandle) -> Result<serde_json::Value, String> {
	crate::IPC::WindAdvancedSync::mountain_get_sync_status(app_handle)
		.await
		.map_err(|e: String| {
			error!("[IPC] [Sync] Failed to get sync status: {}", e);
			e.to_string()
		})
		.map(|status| serde_json::to_value(status).unwrap_or(serde_json::Value::Null))
}

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
		.map_err(|e: String| {
			error!("[IPC] [Sync] Failed to subscribe to updates: {}", e);
			e.to_string()
		})
		.map(|_| serde_json::Value::Null)
}

// =============================================================================
// Binary Entrypoint
// =============================================================================

/// The main function that orchestrates the application lifecycle.
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
		// [Boot] [Args] CLI parsing (using CliParse module)
		// ---------------------------------------------------------------------
		let WorkspaceConfigurationPath = CliParseFn();
		let InitialFolders = vec![];

		// ---------------------------------------------------------------------
		// [Boot] [State] ApplicationState (using StateBuild module)
		// ---------------------------------------------------------------------
		debug!("[Boot] [State] Building ApplicationState...");

		let AppState = BuildStateFn(InitialFolders, WorkspaceConfigurationPath);

		debug!(
			"[Boot] [State] ApplicationState created with {} workspace folders.",
			AppState.WorkspaceFolders.lock().map(|f| f.len()).unwrap_or(0)
		);

		// ---------------------------------------------------------------------
		// [Boot] [Runtime] Scheduler handles (using RuntimeBuild module)
		// ---------------------------------------------------------------------
		let Scheduler = Arc::new(SchedulerBuilder::Create().Build());
		let SchedulerForClosure = Scheduler.clone();
		TraceStep!("[Boot] [Echo] Scheduler handles prepared.");

		// ---------------------------------------------------------------------
		// [Boot] [Localhost] Port selection (using PortSelector module)
		// ---------------------------------------------------------------------
		let ServerPort = SelectPort();
		let LocalhostUrl = BuildPortUrl(ServerPort);

		// ---------------------------------------------------------------------
		// [Boot] [Logging] Log level resolution (using LogLevel module)
		// ---------------------------------------------------------------------
		let log_level = ResolveLogLevel();

		// ---------------------------------------------------------------------
		// [Boot] [Tauri] Builder setup (using TauriBuild module)
		// ---------------------------------------------------------------------
		let mut Builder = TauriBuildFn();

		Builder
			.plugin(LoggingPluginFn(log_level))
			.plugin(LocalhostPluginFn(ServerPort))
			.manage(AppState.clone())
		.setup({
			let LocalhostUrl = LocalhostUrl.clone();
			move |app: &mut App| {
				info!("[Lifecycle] [Setup] Setup hook started.");
				debug!("[Lifecycle] [Setup] LocalhostUrl={}", LocalhostUrl);
	
				let AppHandle = app.handle().clone();
				TraceStep!("[Lifecycle] [Setup] AppHandle acquired.");

					// ---------------------------------------------------------
					// [UI] [Tray] Initialize System Tray
					// ---------------------------------------------------------
					debug!("[UI] [Tray] Initializing system tray...");
					if let Err(Error) = EnableTrayFn::enable_tray(app) {
						error!("[UI] [Tray] Failed to enable tray: {}", Error);
					}

					// ---------------------------------------------------------
					// [Lifecycle] [Commands] Register native commands
					// ---------------------------------------------------------
					debug!("[Lifecycle] [Commands] Registering native commands...");
					if let Err(e) = CommandRegisterFn(&AppHandle, &AppState) {
						error!("[Lifecycle] [Commands] Failed to register commands: {}", e);
					}
					debug!("[Lifecycle] [Commands] Native commands registered.");

					// ---------------------------------------------------------
					// [Lifecycle] [IPC] Initialize IPC Server
					// ---------------------------------------------------------
					debug!("[Lifecycle] [IPC] Initializing Mountain IPC Server...");
					if let Err(e) = IPCServerRegisterFn(&AppHandle) {
						error!("[Lifecycle] [IPC] Failed to register IPC server: {}", e);
					}

					// ---------------------------------------------------------
					// [UI] [Window] Build main window
					// ---------------------------------------------------------
					debug!("[UI] [Window] Building main window...");
					let MainWindow = WindowBuildFn(app, LocalhostUrl.clone());
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
					let Environment = Arc::new(MountainEnvironment::Create(AppHandle.clone()));
					info!("[Backend] [Env] MountainEnvironment ready.");

					// ---------------------------------------------------------
					// [Backend] [Runtime] ApplicationRunTime
					// ---------------------------------------------------------
					debug!("[Backend] [Runtime] Creating ApplicationRunTime...");
					let Runtime = Arc::new(ApplicationRunTime::Create(SchedulerForClosure.clone(), Environment.clone()));
					AppHandle.manage(Runtime.clone());
					info!("[Backend] [Runtime] ApplicationRunTime managed.");

					// ---------------------------------------------------------
					// [Lifecycle] [IPC] Initialize Status Reporter
					// ---------------------------------------------------------
					if let Err(e) = StatusReporterRegisterFn(&AppHandle, Runtime.clone()) {
						error!("[Lifecycle] [IPC] Failed to initialize status reporter: {}", e);
					}

					// ---------------------------------------------------------
					// [Lifecycle] [IPC] Initialize Advanced Features
					// ---------------------------------------------------------
					if let Err(e) = AdvancedFeaturesRegisterFn(&AppHandle, Runtime.clone()) {
						error!("[Lifecycle] [IPC] Failed to initialize advanced features: {}", e);
					}

					// ---------------------------------------------------------
					// [Lifecycle] [IPC] Initialize Wind Advanced Sync
					// ---------------------------------------------------------
					if let Err(e) = WindSyncRegisterFn(&AppHandle, Runtime.clone()) {
						error!("[Lifecycle] [IPC] Failed to initialize wind advanced sync: {}", e);
					}

					// ---------------------------------------------------------
					// [Lifecycle] [PostSetup] Async initialization work
					// ---------------------------------------------------------
					let PostSetupAppHandle = AppHandle.clone();
					let PostSetupEnvironment = Environment.clone();

					tauri::async_runtime::spawn(async move {
						info!("[Lifecycle] [PostSetup] Starting...");
						let AppStateForSetup = PostSetupEnvironment.ApplicationState.clone();
						TraceStep!("[Lifecycle] [PostSetup] AppState cloned.");

						// [Config]
						ConfigurationInitializeFn(&PostSetupEnvironment).await;

						// [Extensions] [ScanPaths]
						ScanPathConfigureFn(&AppStateForSetup);

						// [Extensions] [Scan]
						ExtensionPopulateFn(PostSetupAppHandle.clone(), &AppStateForSetup).await;

						// [Vine] [gRPC]
						VineStartFn(PostSetupAppHandle.clone(), "[::1]:50051".to_string(), "[::1]:50052".to_string())
							.await;

						// [Cocoon] [Sidecar]
						CocoonStartFn(&PostSetupAppHandle, &PostSetupEnvironment).await;

						info!("[Lifecycle] [PostSetup] Complete. System ready.");
					});

					Ok(())
				}
			})
			.plugin(tauri_plugin_dialog::init())
			.plugin(tauri_plugin_fs::init())
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
				DispatchLogic::DispatchFrontendCommand,
				DispatchLogic::ResolveUIRequest,
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
			.build(tauri::generate_context!())
			.expect("FATAL: Error while building Mountain Tauri application")
			.run(move |app_handle: &tauri::AppHandle, event: tauri::RunEvent| {
				// Debug-only: log selected lifecycle events
				if cfg!(debug_assertions) {
					match &event {
						RunEvent::MainEventsCleared => {},
						RunEvent::WindowEvent { .. } => {},
						_ => debug!("[Lifecycle] [RunEvent] {:?}", event),
					}
				}
	
				if let RunEvent::ExitRequested { api, .. } = event {
					warn!("[Lifecycle] [Shutdown] Exit requested. Starting graceful shutdown...");
					api.prevent_exit();
	
					let SchedulerHandle = Scheduler.clone();
					let app_handle_clone = app_handle.clone();
	
					tokio::spawn(async move {
						debug!("[Lifecycle] [Shutdown] Shutting down ApplicationRunTime...");
						let _ = RuntimeShutdownFn(&app_handle_clone).await;
	
						debug!("[Lifecycle] [Shutdown] Stopping Echo scheduler...");
						let _ = SchedulerShutdownFn(SchedulerHandle).await;
	
						info!("[Lifecycle] [Shutdown] Done. Exiting process.");
						app_handle_clone.exit(0);
					});
				}
			});

		info!("[Lifecycle] [Exit] Mountain application has shut down.");
	});
}
