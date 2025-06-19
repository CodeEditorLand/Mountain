//! # InitializationData
//!
//! Contains the logic for constructing the `IExtensionHostInitData` payload,
//! which is sent to the `Cocoon` sidecar during the initial handshake to
//! bootstrap its state.

use serde_json::{Value, json};
use tauri::AppHandle;

use crate::ApplicationState::{
	ApplicationState::ApplicationState,
	DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
};

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
	let WorkSpaceStorage = AppData.join("User/workspaceStorage");

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
			"workspaceStorageHome": WorkSpaceStorage,
			"extensionDevelopmentLocationURI": [],
			"extensionTestsLocationURI": Value::Null,
			"extensionLogLevel": [["info", "Default"]],
		},

		// --- WorkSpace & Remote ---
		"workspace": WorkSpaceDTO,
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
