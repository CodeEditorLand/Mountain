// File: Mountain/Source/ProcessManagement/InitializationData.rs
// Role: Constructs initial data payloads for the `Sky` frontend and `Cocoon`
// sidecar. Responsibilities:
//   - `ConstructSandboxConfiguration`: Gathers host environment data for the
//     frontend.
//   - `ConstructExtensionHostInitializationData`: Assembles all necessary data
//     for the extension host to initialize, including extensions, workspace
//     info, and paths.

//! # InitializationData
//!
//! Contains the logic for constructing the initial data payloads that are sent
//! to the `Sky` frontend and the `Cocoon` sidecar to bootstrap their states.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, env, sync::Arc};

use Common::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
	WorkSpace::WorkSpaceProvider::WorkSpaceProvider,
};
use log::info;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry};
use uuid::Uuid;

use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
};

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

	let BackupPath = AppDataDir.join("Backups").join(ApplicationState.GetWorkSpaceIdentifier()?);

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

	Ok(json!({
		"windowId": ApplicationHandle.get_webview_window("main").unwrap().label(),

		// TODO: Persist and read from storage
		"machineId": Uuid::new_v4().to_string(),

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

	let WorkspaceProvider:Arc<dyn WorkSpaceProvider> = Environment.Require();

	let WorkspaceName = WorkspaceProvider
		.GetWorkSpaceName()
		.await?
		.unwrap_or_else(|| "Mountain WorkSpace".to_string());

	let WorkSpaceFoldersGuard = ApplicationState.WorkSpaceFolders.lock().unwrap();

	let WorkSpaceDTO = if WorkSpaceFoldersGuard.is_empty() {
		Value::Null
	} else {
		json!({

			"id": ApplicationState.GetWorkSpaceIdentifier()?,

			"name": WorkspaceName,

			"configuration": ApplicationState.WorkSpaceConfigurationPath.lock().unwrap().as_ref().map(|p| p.to_string_lossy()),

			"isUntitled": ApplicationState.WorkSpaceConfigurationPath.lock().unwrap().is_none(),

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

	let WorkSpaceStorage = AppData.join("User/workspaceStorage");

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

			"workspaceStorageHome": url::Url::from_directory_path(WorkSpaceStorage).unwrap(),

			"extensionDevelopmentLocationURI": [],

			"extensionTestsLocationURI": Value::Null,

			"extensionLogLevel": [["info", "Default"]],

		},

		"workspace": WorkSpaceDTO,

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

			"machineId": Uuid::new_v4().to_string(),

			"firstSessionDate": "2024-01-01T00:00:00.000Z",

			"msftInternal": false
		},

		"extensions": ExtensionsDTO,

		"autoStart": true,

		// UIKind.Desktop
		"uiKind": 1,
	}))
}
