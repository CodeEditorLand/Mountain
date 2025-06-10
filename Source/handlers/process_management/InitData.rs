use log::Level;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

/// @module InitData (ProcessManagement/Handlers)
/// @description Contains the logic for constructing the
/// `IExtensionHostInitData` payload, which is sent to the Cocoon sidecar during
/// the initial handshake.
use crate::AppState::{AppState::AppState, Dto::*};

/// Constructs the full `IExtensionHostInitData` DTO with high fidelity,
/// mirroring the payload created by VS Code's `localProcessExtensionHost.ts`.
///
/// This function gathers data from the central `AppState` and Tauri's
/// `AppHandle` to create a comprehensive snapshot of the application's state
/// and configuration for the extension host.
///
/// @param AppHandle - The Tauri application handle.
/// @param AppStateInstance - A reference to the application's central state.
/// @returns A `serde_json::Value` representing the complete initialization
/// payload.
pub fn ConstructExtensionHostInitData<R:Runtime>(AppHandle:&AppHandle<R>, AppStateInstance:&AppState) -> Value {
	let ExtensionsGuard = AppStateInstance.ScannedExtensions.lock().unwrap();
	let ExtensionsDto:Vec<&ExtensionDescriptionStateDto> = ExtensionsGuard.values().collect();

	let WorkspaceFoldersGuard = AppStateInstance.WorkspaceFolders.lock().unwrap();
	let WorkspaceDto = if WorkspaceFoldersGuard.is_empty() {
		Value::Null
	} else {
		json!({
			"id": AppStateInstance.GetWorkspaceIdentifier().unwrap_or_default(),
			"name": AppStateInstance.GetWorkspaceName().unwrap_or_default(),
			"configuration": AppStateInstance.WorkspaceConfigurationPath.lock().unwrap().as_ref().map(|p| p.to_string_lossy()),
			"isUntitled": AppStateInstance.WorkspaceConfigurationPath.lock().unwrap().is_none(),
			"transient": false
		})
	};

	json!({
		// --- Application Info ---
		"commit": "dev-commit-hash",
		"version": "1.0.0",
		"quality": "development",
		"parentPid": std::process::id(),

		// --- Environment Info ---
		"environment": {
			"isExtensionDevelopmentDebug": false,
			"appName": "Land",
			"appHost": "desktop",
			"appUriScheme": "land",
			"appLanguage": "en",
			"isExtensionTelemetryLoggingOnly": true,
			"appRoot": AppHandle.path_resolver().app_dir().map(|p| p.to_string_lossy().to_string()),
			"globalStorageHome": AppHandle.path_resolver().app_config_dir().unwrap().join("User/globalStorage"),
			"workspaceStorageHome": AppHandle.path_resolver().app_config_dir().unwrap().join("User/workspaceStorage"),
			"extensionDevelopmentLocationURI": [],
			"extensionTestsLocationURI": Value::Null,
			"extensionLogLevel": [["info", "Default"]], // TODO: Populate from config
		},

		// --- Workspace & Remote ---
		"workspace": WorkspaceDto,
		"remote": {
			"isRemote": false,
			"authority": Value::Null,
			"connectionData": Value::Null,
		},

		// --- Logging & Telemetry ---
		"consoleForward": { "includeStack": true, "logNative": true },
		"logLevel": log::max_level() as u32, // Map log::Level to VS Code's LogLevel enum
		"logsLocation": AppHandle.path_resolver().app_log_dir().unwrap_or_default(),
		"telemetryInfo": {
			"sessionId": AppHandle.try_state::<uuid::Uuid>().map(|id| id.to_string()).unwrap_or_default(),
			"machineId": "dev-machine-id", // TODO: Implement stable machine ID
			"firstSessionDate": "dev-first-session-date",
			"msftInternal": false
		},

		// --- Extensions & Startup ---
		"extensions": ExtensionsDto,
		"autoStart": true,
		"uiKind": 1, // Corresponds to UIKind.Desktop
	})
}
