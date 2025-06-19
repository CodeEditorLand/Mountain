//! # InitializationData
//!
//! Contains the logic for constructing the `IExtensionHostInitData` payload,
//! which is sent to the `Cocoon` sidecar during the initial handshake to
//! bootstrap its state.

use log::Level;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

use crate::ApplicationState::{ApplicationState::ApplicationState, DTO::*};

/// Constructs the full `IExtensionHostInitData` DTO with high fidelity,
/// mirroring the payload created by VS Code's `localProcessExtensionHost.ts`.
///
/// This function gathers data from the central `ApplicationState` and Tauri's
/// `AppHandle` to create a comprehensive snapshot of the application's state
/// and configuration for the extension host.
///
/// # Parameters
/// * `ApplicationHandle`: The Tauri application handle.
/// * `ApplicationState`: A reference to the application's central state.
///
/// # Returns
/// A `serde_json::Value` representing the complete initialization payload.
pub fn ConstructExtensionHostInitializationData(
	ApplicationHandle:&AppHandle,
	ApplicationState:&ApplicationState,
) -> Value {
	let ExtensionsGuard = ApplicationState.ScannedExtensions.lock().unwrap();
	let ExtensionsDTO:Vec<&ExtensionDescriptionStateDTO> = ExtensionsGuard.values().collect();

	let WorkspaceFoldersGuard = ApplicationState.WorkspaceFolders.lock().unwrap();
	let WorkspaceDTO = if WorkspaceFoldersGuard.is_empty() {
		Value::Null
	} else {
		json!({
			"id": ApplicationState.GetWorkspaceIdentifier().unwrap_or_default(),
			"name": "TODO: GetWorkspaceName", // Placeholder
			"configuration": ApplicationState.WorkspaceConfigurationPath.lock().unwrap().as_ref().map(|p| p.to_string_lossy()),
			"isUntitled": ApplicationState.WorkspaceConfigurationPath.lock().unwrap().is_none(),
			"transient": false
		})
	};

	let AppRoot = ApplicationHandle.path_resolver().app_dir().unwrap_or_default();
	let AppData = ApplicationHandle
		.path_resolver()
		.app_data_dir()
		.unwrap_or_else(|| AppRoot.join(".appdata"));
	let LogsLocation = ApplicationHandle
		.path_resolver()
		.app_log_dir()
		.unwrap_or_else(|| AppData.join("logs"));
	let GlobalStorage = AppData.join("User/globalStorage");
	let WorkspaceStorage = AppData.join("User/workspaceStorage");

	json!({
		// --- Application Info ---
		"commit": "dev-commit-hash",
		"version": "1.0.0",
		"quality": "development",
		"parentPid": std::process::id(),

		// --- Environment Info ---
		"Environment": {
			"isExtensionDevelopmentDebug": false,
			"appName": "Mountain",
			"appHost": "desktop",
			"appUriScheme": "mountain",
			"appLanguage": "en",
			"isExtensionTelemetryLoggingOnly": true,
			"appRoot": AppRoot,
			"globalStorageHome": GlobalStorage,
			"workspaceStorageHome": WorkspaceStorage,
			"extensionDevelopmentLocationURI": [],
			"extensionTestsLocationURI": Value::Null,
			"extensionLogLevel": [["info", "Default"]],
		},

		// --- Workspace & Remote ---
		"workspace": WorkspaceDTO,
		"remote": {
			"isRemote": false,
			"authority": Value::Null,
			"connectionData": Value::Null,
		},

		// --- Logging & Telemetry ---
		"consoleForward": { "includeStack": true, "logNative": true },
		"logLevel": log::max_level().to_string(),
		"logsLocation": LogsLocation,
		"telemetryInfo": {
			"sessionId": "dev-session-id", // Placeholder
			"machineId": "dev-machine-id", // Placeholder
			"firstSessionDate": "dev-first-session-date",
			"msftInternal": false
		},

		// --- Extensions & Startup ---
		"extensions": ExtensionsDTO,
		"autoStart": true,
		"uiKind": 1, // Corresponds to UIKind.Desktop
	})
}
