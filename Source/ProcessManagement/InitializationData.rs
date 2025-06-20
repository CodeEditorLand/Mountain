//! # InitializationData
//!
//! Contains the logic for constructing the `IExtensionHostInitData` payload,
//! which is sent to the `Cocoon` sidecar during the initial handshake to
//! bootstrap its state.

use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

use crate::ApplicationState::{
	ApplicationState::ApplicationState,
	DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
};

/// Constructs the full `IExtensionHostInitData` DTO with high fidelity,
/// mirroring the payload created by VS Code's `localProcessExtensionHost.ts`.
pub fn ConstructExtensionHostInitializationData(
	ApplicationHandle:&AppHandle,
	ApplicationState:&ApplicationState,
) -> Value {
	let ExtensionsGuard = ApplicationState.ScannedExtensions.lock().unwrap();
	let ExtensionsDTO:Vec<&ExtensionDescriptionStateDTO> = ExtensionsGuard.values().collect();

	let WorkSpaceFoldersGuard = ApplicationState.WorkSpaceFolders.lock().unwrap();
	let WorkSpaceDTO = if WorkSpaceFoldersGuard.is_empty() {
		Value::Null
	} else {
		json!({
			"id": ApplicationState.GetWorkSpaceIdentifier().unwrap_or_default(),
			"name": "TODO: GetWorkSpaceName", // Placeholder
			"configuration": ApplicationState.WorkSpaceConfigurationPath.lock().unwrap().as_ref().map(|p| p.to_string_lossy()),
			"isUntitled": ApplicationState.WorkSpaceConfigurationPath.lock().unwrap().is_none(),
			"transient": false
		})
	};

	let path_resolver = ApplicationHandle.path();
	let AppRoot = path_resolver.resource_dir().unwrap_or_default();
	let AppData = path_resolver.app_data_dir().unwrap_or_else(|_| AppRoot.join(".appdata"));
	let LogsLocation = path_resolver.app_log_dir().unwrap_or_else(|_| AppData.join("logs"));
	let GlobalStorage = AppData.join("User/globalStorage");
	let WorkSpaceStorage = AppData.join("User/workspaceStorage");

	json!({
		"commit": "dev-commit-hash",
		"version": "1.0.0",
		"quality": "development",
		"parentPid": std::process::id(),
		"Environment": {
			"isExtensionDevelopmentDebug": false,
			"appName": "Mountain",
			"appHost": "desktop",
			"appUriScheme": "mountain",
			"appLanguage": "en",
			"isExtensionTelemetryLoggingOnly": true,
			"appRoot": AppRoot,
			"globalStorageHome": GlobalStorage,
			"workspaceStorageHome": WorkSpaceStorage,
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
		"logLevel": log::max_level().to_string(),
		"logsLocation": LogsLocation,
		"telemetryInfo": {
			"sessionId": "dev-session-id",
			"machineId": "dev-machine-id",
			"firstSessionDate": "dev-first-session-date",
			"msftInternal": false
		},
		"extensions": ExtensionsDTO,
		"autoStart": true,
		"uiKind": 1,
	})
}
