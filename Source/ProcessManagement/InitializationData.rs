//! # InitializationData (ProcessManagement)
//!
//! Constructs the initial data payloads that are sent to the `Sky` frontend
//! and the `Cocoon` sidecar to bootstrap their states during application
//! startup.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Frontend Sandbox Configuration
//! - Gather host environment data (paths, platform, versions)
//! - Construct `ISandboxConfiguration` payload for Sky
//! - Include machine ID, session ID, and user environment
//! - Provide appRoot, homeDir, tmpDir, and userDataDir URIs
//!
//! ### 2. Extension Host Initialization
//! - Assemble data for extension host (Cocoon) startup
//! - Include discovered extensions list
//! - Provide workspace information (folders, configuration)
//! - Set up storage paths (globalStorage, workspaceStorage)
//! - Configure logging and telemetry settings
//!
//! ### 3. Path Resolution
//! - Resolve application root from Tauri resources
//! - Resolve app data directory for persistence
//! - Resolve home directory and temp directory
//! - Handle path errors with descriptive `CommonError` types
//!
//! ## ARCHITECTURAL ROLE
//!
//! InitializationData is the **bootstrap orchestrator** for Mountain's
//! startup sequence:
//!
//! ```text
//! Binary::Main ──► InitializationData ──► Sky (Frontend)
//! │
//! └─► Cocoon (Extension Host)
//! ```
//!
//! ### Position in Mountain
//! - `ProcessManagement` module: Process lifecycle and initialization
//! - Called during `Binary::Main` startup and `CocoonManagement` initialization
//! - Provides complete environment snapshot for all processes
//!
//! ### Dependencies
//! - `tauri::AppHandle`: Path resolution and package info
//! - `CommonLibrary::Environment::Requires`: DI for services
//! - `CommonLibrary::Error::CommonError`: Error handling
//! - `uuid::Uuid`: Generate machine/session IDs
//! - `serde_json::json`: Payload construction
//!
//! ### Dependents
//! - `Binary::Main::Fn`: Calls `ConstructSandboxConfiguration` for UI
//! - `CocoonManagement::InitializeCocoon`: Calls
//!   `ConstructExtensionHostInitializationData`
//!
//! ## PAYLOAD FORMATS
//!
//! ### ISandboxConfiguration (for Sky)
//! ```json
//! {
//!   "windowId": "main",
//!   "machineId": "uuid",
//!   "sessionId": "uuid",
//!   "logLevel": 2,
//!   "userEnv": { ... },
//!   "appRoot": "file:///...",
//!   "appName": "Mountain",
//!   "platform": "darwin|win32|linux",
//!   "arch": "x64|arm64",
//!   "versions": { "mountain": "x.y.z", "electron": "0.0.0-tauri", ... },
//!   "homeDir": "file:///...",
//!   "tmpDir": "file:///...",
//!   "userDataDir": "file:///...",
//!   "backupPath": "file:///...",
//!   "productConfiguration": { ... }
//! }
//! ```
//!
//! ### IExtensionHostInitData (for Cocoon)
//! ```json
//! {
//!   "commit": "dev-commit-hash",
//!   "version": "x.y.z",
//!   "parentPid": 12345,
//!   "environment": {
//!     "appName": "Mountain",
//!     "appRoot": "file:///...",
//!     "globalStorageHome": "file:///...",
//!     "workspaceStorageHome": "file:///...",
//!     "extensionLogLevel": [["info", "Default"]]
//!   },
//!   "workspace": { "id": "...", "name": "...", ... },
//!   "logsLocation": "file:///...",
//!   "telemetryInfo": { ... },
//!   "extensions": [ ... ],
//!   "autoStart": true,
//!   "uiKind": 1
//! }
//! ```
//!
//! ## ERROR HANDLING
//!
//! - Path resolution failures return `CommonError::ConfigurationLoad`
//! - Workspace identifier errors propagate from
//!   `ApplicationState::GetWorkspaceIdentifier`
//! - JSON serialization errors should not occur (using `json!` macro)
//!
//! ## PLATFORM DETECTION
//!
//! Platform strings match VS Code conventions:
//! - `"win32"` for Windows
//! - `"darwin"` for macOS
//! - `"linux"` for Linux
//!
//! Architecture mapping:
//! - `"x64"` for x86_64
//! - `"arm64"` for aarch64
//! - `"ia32"` for x86
//!
//! ## TODO
//!
//! - [ ] Persist machineId across sessions (currently generated new each
//!   launch)
//! - [ ] Add environment variable overrides for development
//! - [ ] Implement workspace cache for faster startup
//! - [ ] Add telemetry for initialization performance
//! - [ ] Support remote workspace URIs
//!
//! ## MODULE CONTENTS
//!
//! - [`ConstructSandboxConfiguration`]: Build ISandboxConfiguration for Sky
//! - [`ConstructExtensionHostInitializationData`]: Build IExtensionHostInitData
//!   for Cocoon

use std::{collections::HashMap, env, fs, path::PathBuf, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
	Workspace::WorkspaceProvider::WorkspaceProvider,
};
use log::info;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry};
use uuid::Uuid;

use crate::{ApplicationState::ApplicationState, Environment::MountainEnvironment::MountainEnvironment};

/// Loads or generates a persistent machine ID.
///
/// The machine ID is stored in the app data directory as a simple text file.
/// If the file doesn't exist, a new UUID is generated and saved.
///
/// # Arguments
/// * `app_data_dir` - The application data directory path
///
/// # Returns
/// The machine ID as a String
fn get_or_generate_machine_id(app_data_dir:&PathBuf) -> String {
	let machine_id_path = app_data_dir.join("machine-id.txt");

	// Try to load existing machine ID
	if let Ok(content) = fs::read_to_string(&machine_id_path) {
		let trimmed = content.trim();
		if !trimmed.is_empty() {
			log::info!("[InitializationData] Loaded existing machine ID from disk");
			return trimmed.to_string();
		}
	}

	// Generate and save new machine ID
	let new_machine_id = Uuid::new_v4().to_string();

	// Ensure directory exists
	if let Some(parent) = machine_id_path.parent() {
		if let Err(e) = fs::create_dir_all(parent) {
			log::warn!("[InitializationData] Failed to create machine ID directory: {}", e);
		}
	}

	// Save to disk
	if let Err(e) = fs::write(&machine_id_path, &new_machine_id) {
		log::warn!("[InitializationData] Failed to persist machine ID to disk: {}", e);
	} else {
		log::info!("[InitializationData] Generated and persisted new machine ID");
	}

	new_machine_id
}

/// Constructs the `ISandboxConfiguration` payload needed by the `Sky` frontend.
pub async fn ConstructSandboxConfiguration(
	ApplicationHandle:&AppHandle<Wry>,

	ApplicationState:&Arc<ApplicationState>,
) -> Result<Value, CommonError> {
	info!("[InitializationData] Constructing ISandboxConfiguration for Sky.");

	let PathResolver = ApplicationHandle.path();

	let AppRootUri = PathResolver.resource_dir().map_err(|Error| {
		CommonError::ConfigurationLoad {
			Description:format!("Failed to resolve resource directory (app root): {}", Error),
		}
	})?;

	let AppDataDir = PathResolver.app_data_dir().map_err(|Error| {
		CommonError::ConfigurationLoad { Description:format!("Failed to resolve app data directory: {}", Error) }
	})?;

	let HomeDir = PathResolver.home_dir().map_err(|Error| {
		CommonError::ConfigurationLoad { Description:format!("Failed to resolve home directory: {}", Error) }
	})?;

	let TmpDir = env::temp_dir();

	let BackupPath = AppDataDir.join("Backups").join(ApplicationState.GetWorkspaceIdentifier()?);

	let Platform = match env::consts::OS {
		"windows" => "win32",

		"macos" => "darwin",

		"linux" => "linux",

		_ => "unknown",
	};

	let Arch = match env::consts::ARCH {
		"x86_64" => "x64",

		"aarch64" => "arm64",

		"x86" => "ia32",

		_ => "unknown",
	};

	let Versions = json!({
		"mountain": ApplicationHandle.package_info().version.to_string(),

		// Explicitly signal we are not in Electron
		"electron": "0.0.0-tauri",

		// Representative version
		"chrome": "120.0.0.0",

		// Representative version
		"node": "18.18.2"
	});

	// Load or generate persistent machine ID
	let machine_id = get_or_generate_machine_id(&AppDataDir);

	Ok(json!({
		"windowId": ApplicationHandle.get_webview_window("main").unwrap().label(),

		// Persist the machineId to ApplicationState or persistent storage and load
		// it on subsequent runs. A stable machine identifier is crucial for licensing
		// validation, telemetry deduplication, and cross-session state consistency.
		// Now implemented with persistent storage in app data directory.
		"machineId": machine_id,

		"sessionId": Uuid::new_v4().to_string(),

		"logLevel": log::max_level() as i32,

		"userEnv": env::vars().collect::<HashMap<_,_>>(),

		"appRoot": url::Url::from_directory_path(AppRootUri).unwrap().to_string(),

		"appName": ApplicationHandle.package_info().name.clone(),

		"appUriScheme": "mountain",

		"appLanguage": "en",

		"appHost": "desktop",

		"platform": Platform,

		"arch": Arch,

		"versions": Versions,

		"execPath": env::current_exe().unwrap_or_default().to_string_lossy(),

		"homeDir": url::Url::from_directory_path(HomeDir).unwrap().to_string(),

		"tmpDir": url::Url::from_directory_path(TmpDir).unwrap().to_string(),

		"userDataDir": url::Url::from_directory_path(AppDataDir).unwrap().to_string(),

		"backupPath": url::Url::from_directory_path(BackupPath).unwrap().to_string(),

		"nls": { "messages": {}, "language": "en", "availableLanguages": { "en": "English" } },

		"productConfiguration": {

			"nameShort": "Mountain",

			"nameLong": "Mountain Editor",

			"applicationName": "mountain",

			"embedderIdentifier": "mountain-desktop"
		},

		"resourcesPath": PathResolver.resource_dir().unwrap_or_default().to_string_lossy(),

		"VSCODE_CWD": env::current_dir().unwrap_or_default().to_string_lossy(),
	}))
}

/// Constructs the `IExtensionHostInitData` payload sent to `Cocoon`.
pub async fn ConstructExtensionHostInitializationData(Environment:&MountainEnvironment) -> Result<Value, CommonError> {
	info!("[InitializationData] Constructing IExtensionHostInitData for Cocoon.");

	let ApplicationState = &Environment.ApplicationState;

	let ApplicationHandle = &Environment.ApplicationHandle;

	let ExtensionManagementProvider:Arc<dyn ExtensionManagementService> = Environment.Require();

	let ExtensionsDTO = ExtensionManagementProvider.GetExtensions().await?;

	let WorkspaceProvider:Arc<dyn WorkspaceProvider> = Environment.Require();

	let WorkspaceName = WorkspaceProvider
		.GetWorkspaceName()
		.await?
		.unwrap_or_else(|| "Mountain Workspace".to_string());

	let WorkspaceFoldersGuard = ApplicationState.Workspace.WorkspaceFolders.lock().unwrap();

	let WorkspaceDTO = if WorkspaceFoldersGuard.is_empty() {
		Value::Null
	} else {
		json!({

			"id": ApplicationState.GetWorkspaceIdentifier()?,

			"name": WorkspaceName,

			"configuration": ApplicationState.Workspace.WorkspaceConfigurationPath.lock().unwrap().as_ref().map(|p| p.to_string_lossy()),

			"isUntitled": ApplicationState.Workspace.WorkspaceConfigurationPath.lock().unwrap().is_none(),

			"transient": false
		})
	};

	let PathResolver = ApplicationHandle.path();

	let AppRoot = PathResolver
		.resource_dir()
		.map_err(|Error| CommonError::ConfigurationLoad { Description:Error.to_string() })?;

	let AppData = PathResolver
		.app_data_dir()
		.map_err(|Error| CommonError::ConfigurationLoad { Description:Error.to_string() })?;

	let LogsLocation = PathResolver
		.app_log_dir()
		.map_err(|Error| CommonError::ConfigurationLoad { Description:Error.to_string() })?;

	let GlobalStorage = AppData.join("User/globalStorage");

	let WorkspaceStorage = AppData.join("User/workspaceStorage");

	Ok(json!({

		"commit": "dev-commit-hash",

		"version": ApplicationHandle.package_info().version.to_string(),

		"quality": "development",

		"parentPid": std::process::id(),

		"environment": {

			"isExtensionDevelopmentDebug": false,

			"appName": "Mountain",

			"appHost": "desktop",

			"appUriScheme": "mountain",

			"appLanguage": "en",

			"isExtensionTelemetryLoggingOnly": true,

			"appRoot": url::Url::from_directory_path(AppRoot.clone()).unwrap(),

			"globalStorageHome": url::Url::from_directory_path(GlobalStorage).unwrap(),

			"workspaceStorageHome": url::Url::from_directory_path(WorkspaceStorage).unwrap(),

			"extensionDevelopmentLocationURI": [],

			"extensionTestsLocationURI": Value::Null,

			"extensionLogLevel": [["info", "Default"]],

		},

		"workspace": WorkspaceDTO,

		"remote": {

			"isRemote": false,

			"authority": Value::Null,

			"connectionData": Value::Null,

		},

		"consoleForward": { "includeStack": true, "logNative": true },

		"logLevel": log::max_level() as i32,

		"logsLocation": url::Url::from_directory_path(LogsLocation).unwrap(),

		"telemetryInfo": {

			"sessionId": Uuid::new_v4().to_string(),

			"machineId": get_or_generate_machine_id(&AppData),

			"firstSessionDate": "2024-01-01T00:00:00.000Z",

			"msftInternal": false
		},

		"extensions": ExtensionsDTO,

		"autoStart": true,

		// UIKind.Desktop
		"uiKind": 1,
	}))
}
