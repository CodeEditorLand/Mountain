//! `InitializationData::ConstructSandboxConfiguration`

use std::{
	collections::HashMap,
	env,
	path::PathBuf,
	sync::{Arc, OnceLock},
};
use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
	Workspace::WorkspaceProvider::WorkspaceProvider,
};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry};
use uuid::Uuid;
use crate::{
	ApplicationState::Struct::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	dev_log,
};

static SESSION_ID:OnceLock<String> = OnceLock::new();

/// Constructs the `ISandboxConfiguration` payload needed by the `Sky` frontend.
pub async fn Fn(
	ApplicationHandle:&AppHandle<Wry>,

	ApplicationState:&Arc<ApplicationState>,
) -> Result<Value, CommonError> {
	dev_log!("cocoon", "[InitializationData] Constructing ISandboxConfiguration for Sky.");

	let PathResolver = ApplicationHandle.Path();

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

	// `logsPath` is a required field of `ISandboxConfiguration`. VS Code reads
	// it via `NativeWorkbenchEnvironmentService.logsHome` → `URI.file(logsPath)`.
	// Missing it leaves logsPath=undefined → URI.file(undefined).fsPath=undefined
	// → path.join(undefined,"…") → "The path argument must be of type string".
	let LogsPath = AppDataDir.join("logs").join(crate::IPC::DevLog::SessionTimestamp::Fn());

	let _ = std::fs::create_dir_all(&LogsPath);

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
	let machine_id = GetOrGenerateMachineId(&AppDataDir).await;

	// Build the `profiles` section outside the main `json!` call to avoid
	// exceeding the macro's recursion limit (default 64). Each nested json!
	// call here is shallow enough to compile without issue.
	let UserProfile = AppDataDir.join("User");

	// Helper closure: build a `UriComponents` JSON object for a filesystem path.
	let FileUri = |P:std::path::PathBuf| -> serde_json::Value {
		json!({
			"scheme": "file",
			"authority": "",
			"path": P.to_string_lossy(),
			"query": "",
			"fragment": ""
		})
	};

	let DefaultProfile = json!({
		"id": "__default__profile__",
		"name": "Default",
		"location": FileUri(UserProfile.clone()),
		"isDefault": true,
		"globalStorageHome": FileUri(UserProfile.join("globalStorage")),
		"settingsResource": FileUri(UserProfile.join("settings.json")),
		"keybindingsResource": FileUri(UserProfile.join("keybindings.json")),
		"tasksResource": FileUri(UserProfile.join("tasks.json")),
		"snippetsHome": FileUri(UserProfile.join("snippets")),
		"promptsHome": FileUri(UserProfile.join("prompts")),
		"extensionsResource": FileUri(UserProfile.join("extensions.json")),
		"mcpResource": FileUri(UserProfile.join("mcp.json")),
		"languageModelsResource": FileUri(UserProfile.join("chatLanguageModels.json")),
		"agentPluginsHome": FileUri(UserProfile.join("agent-plugins")),
		"cacheHome": FileUri(UserProfile.join("profiles/.cache/__default__profile__"))
	});

	let ProfilesSection = json!({
		"home": FileUri(UserProfile.join("profiles")),
		"all": [DefaultProfile.clone()],
		"profile": DefaultProfile
	});

	// Pre-build other nested sections that contribute heavily to the token
	// count of the outer json! call and could push it past the limit.
	let NlsSection = json!({
		"messages": {},
		"language": "en",
		"availableLanguages": { "en": "English" }
	});

	let ProductConfig = json!({
		"nameShort": std::env::var("ProductNameShort").unwrap_or_else(|_| "FIDDEE".into()),
		"nameLong": std::env::var("ProductNameLong").unwrap_or_else(|_| "FIDDEE".into()),
		"applicationName": std::env::var("ProductApplicationName").unwrap_or_else(|_| "fiddee".into()),
		"embedderIdentifier": std::env::var("ProductEmbedderIdentifier").unwrap_or_else(|_| "fiddee-desktop".into()),
		"dataFolderName": std::env::var("ProductDataFolderName").unwrap_or_else(|_| ".fiddee".into()),
		"sharedDataFolderName": std::env::var("ProductDataFolderName").unwrap_or_else(|_| ".fiddee".into()),
		"version": std::env::var("ProductVersion").unwrap_or_else(|_| "1.0.0".into()),
	});

	let OsSection = json!({
		"release": "22.0.0",
		"hostname": "land",
		"arch": env::consts::ARCH,
	});

	Ok(json!({
		"windowId": ApplicationHandle.get_webview_window("main").unwrap().label(),

		// Persist the machineId to ApplicationState or persistent storage and load
		// it on subsequent runs. A stable machine identifier is crucial for licensing
		// validation, telemetry deduplication, and cross-session state consistency.
		// Now implemented with persistent storage in app data directory.
		"machineId": machine_id,

		"sessionId": SessionId(),

		"logLevel": log::max_level() as i32,

		"userEnv": env::vars().collect::<HashMap<_,_>>(),

		// `INativeWindowConfiguration.appRoot` - plain OS filesystem path.
		// VS Code's `AbstractNativeEnvironmentService.appRoot` returns this
		// string directly and passes it to `path.join(appRoot, ...)`.
		// Previously sent as a `file://` URL which caused `URI.file(fileUrl)`
		// to construct a URI with path `/file:///…` (double-scheme), making
		// every downstream `path.join` operate on a malformed base.
		"appRoot": AppRootUri.to_string_lossy(),

		"appName": ApplicationHandle.package_info().name.clone(),

		"appUriScheme": "mountain",

		"appLanguage": "en",

		"appHost": "desktop",

		"platform": Platform,

		"arch": Arch,

		"versions": Versions,

		"execPath": env::current_exe().unwrap_or_default().to_string_lossy(),

		// Plain OS paths for all home/data/tmp/backup.
		// VS Code wraps these in `URI.file(path)` and `path.join(path, …)`;
		// both require a real filesystem path, not a `file://` URL string.
		"homeDir": HomeDir.to_string_lossy(),

		"tmpDir": TmpDir.to_string_lossy(),

		"userDataDir": AppDataDir.to_string_lossy(),

		"backupPath": BackupPath.to_string_lossy(),

		"logsPath": LogsPath.to_string_lossy(),

		// Required non-optional fields in INativeWindowConfiguration.
		// Missing these causes crashes in NativeWorkbenchEnvironmentService getters
		// that access them without null-checks.
		"perfMarks": [],

		"colorScheme": { "dark": false, "highContrast": false },

		"loggers": [],

		"mainPid": std::process::id(),

		"os": OsSection,

		"nls": NlsSection,

		// Atom I5: read from process env. Pre-built above to reduce the
		// token count inside this json! call.
		"productConfiguration": ProductConfig,

		"resourcesPath": PathResolver.resource_dir().unwrap_or_default().to_string_lossy(),

		"VSCODE_CWD": env::current_dir().unwrap_or_default().to_string_lossy(),

		// Pre-built outside json! to avoid macro recursion limit (see above).
		"profiles": ProfilesSection,

		// Required non-optional fields added defensively to avoid future
		// workbench crashes on properties accessed without null-checks.
		"sqmId": "",
		"devDeviceId": "",
		"isPortable": false,
	}))
}
