// @module InitializationData (process_management/Handler)
// @description Contains the logic for constructing the
// `IExtensionHostInitData` payload, which is sent to the Cocoon sidecar during
// the initial handshake.

use log::Level;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

use crate::ApplicationState::{ApplicationState::ApplicationState, DTO::*};

/// Constructs the full `IExtensionHostInitData` DTO with high fidelity,
/// mirroring the payload created by VS Code's `localProcessExtensionHost.ts`.
///
/// This function gathers data from the central `ApplicationState` and Tauri's
/// `AppHandle` to create a comprehensive snapshot of the application's state
/// and configuration for the extension host.
///
/// @param app_handle - The Tauri application handle.
/// @param app_state - A reference to the application's central state.
/// @returns A `serde_json::Value` representing the complete initialization
/// payload.
pub fn ConstructExtensionHostInitializationData<R:Runtime>(app_handle:&AppHandle<R>, app_state:&ApplicationState) -> Value {
	let extensions_guard = app_state.ScannedExtensions.lock().unwrap();
	let extensions_DTO:Vec<&ExtensionDescriptionStateDto> = extensions_guard.values().collect();

	let workspace_folders_guard = app_state.WorkspaceFolders.lock().unwrap();
	let workspace_DTO = if workspace_folders_guard.is_empty() {
		Value::Null
	} else {
		json!({
			"id": app_state.GetWorkspaceIdentifier().unwrap_or_default(),
			"name": app_state.GetWorkspaceName().unwrap_or_default(),
			"configuration": app_state.WorkspaceConfigurationPath.lock().unwrap().as_ref().map(|p| p.to_string_lossy()),
			"isUntitled": app_state.WorkspaceConfigurationPath.lock().unwrap().is_none(),
			"transient": false
		})
	};

	let app_root = app_handle.path_resolver().app_dir().unwrap_or_default();
	let app_data = app_handle
		.path_resolver()
		.app_data_dir()
		.unwrap_or_else(|| app_root.join(".appdata"));
	let logs_location = app_handle
		.path_resolver()
		.app_log_dir()
		.unwrap_or_else(|| app_data.join("logs"));
	let global_storage = app_data.join("User/globalStorage");
	let workspace_storage = app_data.join("User/workspaceStorage");

	json!({
		// --- Application Info ---
		"commit": "dev-commit-hash",
		"version": "1.0.0",
		"quality": "development",
		"parentPid": std::process::id(),

		// --- Environment Info ---
		"Environment": {
			"isExtensionDevelopmentDebug": false,
			"appName": "Land",
			"appHost": "desktop",
			"appUriScheme": "land",
			"appLanguage": "en",
			"isExtensionTelemetryLoggingOnly": true,
			"appRoot": app_root,
			"globalStorageHome": global_storage,
			"workspaceStorageHome": workspace_storage,
			"extensionDevelopmentLocationURI": [],
			"extensionTestsLocationURI": Value::Null,
			"extensionLogLevel": [["info", "Default"]], // TODO: Populate from config
		},

		// --- Workspace & Remote ---
		"workspace": workspace_DTO,
		"remote": {
			"isRemote": false,
			"authority": Value::Null,
			"connectionData": Value::Null,
		},

		// --- Logging & Telemetry ---
		"consoleForward": { "includeStack": true, "logNative": true },
		"logLevel": log::max_level().to_string(),
		"logsLocation": logs_location,
		"telemetryInfo": {
			"sessionId": app_handle.try_state::<uuid::Uuid>().map(|id| id.to_string()).unwrap_or_default(),
			"machineId": "dev-machine-id", // TODO: Implement stable machine ID
			"firstSessionDate": "dev-first-session-date",
			"msftInternal": false
		},

		// --- Extensions & Startup ---
		"extensions": extensions_DTO,
		"autoStart": true,
		"uiKind": 1, // Corresponds to UIKind.Desktop
	})
}
