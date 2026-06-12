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

use std::{
	collections::HashMap,
	env,
	path::PathBuf,
	sync::{Arc, OnceLock},
};

/// Stable session ID for the lifetime of this process. Generated once on
/// first access and reused by every call to `ConstructSandboxConfiguration`
/// and `ConstructExtensionHostInitializationData` so extensions comparing
/// `telemetryInfo.sessionId` across calls see the same value.
static SESSION_ID:OnceLock<String> = OnceLock::new();

pub(crate) fn SessionId() -> &'static str { SESSION_ID.get_or_init(|| Uuid::new_v4().to_string()) }

/// Process-lifetime cache for the persistent machine ID. Both
/// `ConstructSandboxConfiguration` and
/// `ConstructExtensionHostInitializationData` need the ID; without the
/// cache the machine-id file is read from disk twice per boot.
static MACHINE_ID:OnceLock<String> = OnceLock::new();

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
	ApplicationState::State::ApplicationState::ApplicationState,
	Environment::MountainEnvironment::MountainEnvironment,
	dev_log,
};

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
async fn get_or_generate_machine_id(app_data_dir:&PathBuf) -> String {
	// Process-lifetime cache: the second caller per boot skips the disk
	// read entirely.
	if let Some(Cached) = MACHINE_ID.get() {
		return Cached.clone();
	}

	let machine_id_path = app_data_dir.join("machine-id.txt");

	// Try to load existing machine ID using async I/O so the Tokio
	// worker thread is not parked during the disk read at boot.
	if let Ok(content) = tokio::fs::read_to_string(&machine_id_path).await {
		let trimmed = content.trim();

		if !trimmed.is_empty() {
			dev_log!("cocoon", "[InitializationData] Loaded existing machine ID from disk");

			return MACHINE_ID.get_or_init(|| trimmed.to_string()).clone();
		}
	}

	// Generate and save new machine ID
	let new_machine_id = Uuid::new_v4().to_string();

	// Ensure directory exists
	if let Some(parent) = machine_id_path.parent() {
		if let Err(e) = tokio::fs::create_dir_all(parent).await {
			dev_log!(
				"cocoon",
				"warn: [InitializationData] Failed to create machine ID directory: {}",
				e
			);
		}
	}

	// Save to disk
	if let Err(e) = tokio::fs::write(&machine_id_path, &new_machine_id).await {
		dev_log!(
			"cocoon",
			"warn: [InitializationData] Failed to persist machine ID to disk: {}",
			e
		);
	} else {
		dev_log!("cocoon", "[InitializationData] Generated and persisted new machine ID");
	}

	MACHINE_ID.get_or_init(|| new_machine_id).clone()
}

/// Constructs the `ISandboxConfiguration` payload needed by the `Sky` frontend.
pub async fn ConstructSandboxConfiguration(
	ApplicationHandle:&AppHandle<Wry>,

	ApplicationState:&Arc<ApplicationState>,
) -> Result<Value, CommonError> {
	dev_log!("cocoon", "[InitializationData] Constructing ISandboxConfiguration for Sky.");

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
	let machine_id = get_or_generate_machine_id(&AppDataDir).await;

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
		"nameShort": std::env::var("ProductNameShort").unwrap_or_else(|_| "FCD".into()),
		"nameLong": std::env::var("ProductNameLong").unwrap_or_else(|_| "FCD".into()),
		"applicationName": std::env::var("ProductApplicationName").unwrap_or_else(|_| "fcd".into()),
		"embedderIdentifier": std::env::var("ProductEmbedderIdentifier").unwrap_or_else(|_| "fcdd".into()),
		"dataFolderName": std::env::var("ProductDataFolderName").unwrap_or_else(|_| ".fcd".into()),
		"sharedDataFolderName": std::env::var("ProductDataFolderName").unwrap_or_else(|_| ".fcd".into()),
		"version": std::env::var("ProductVersion").unwrap_or_else(|_| "1.0.0".into()),
	});

	let OsSection = json!({
		"release": "22.0.0",
		"hostname": "land",
		"arch": env::consts::ARCH,
	});

	Ok(json!({
		// During rapid startup/restart the "main" webview window may not
		// exist yet; fall back to its known label instead of panicking.
		"windowId": ApplicationHandle
			.get_webview_window("main")
			.map(|Window| Window.label().to_string())
			.unwrap_or_else(|| "main".to_string()),

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

		// `ISandboxConfiguration.codeCachePath` - read without a null-guard
		// by the bundled workbench's V8 code-cache loader.
		"codeCachePath": AppDataDir.join("code-cache").to_string_lossy(),

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
		"autoDetectHighContrast": true,
		"accessibilitySupport": false,
		"isInitialStartup": false,
	}))
}

/// Constructs the `IExtensionHostInitData` payload sent to `Cocoon`.
pub async fn ConstructExtensionHostInitializationData(Environment:&MountainEnvironment) -> Result<Value, CommonError> {
	dev_log!("cocoon", "[InitializationData] Constructing IExtensionHostInitData for Cocoon.");

	let ApplicationState = &Environment.ApplicationState;

	let ApplicationHandle = &Environment.ApplicationHandle;

	let ExtensionManagementProvider:Arc<dyn ExtensionManagementService> = Environment.Require();

	let ExtensionsDTO = ExtensionManagementProvider.GetExtensions().await?;

	let WorkspaceProvider:Arc<dyn WorkspaceProvider> = Environment.Require();

	let WorkspaceName = WorkspaceProvider
		.GetWorkspaceName()
		.await?
		.unwrap_or_else(|| "Mountain Workspace".to_string());

	// Scope the MutexGuard so it is dropped before any `.await` below.
	// `MutexGuard<T>` is not `Send`; holding it across an await makes the
	// future non-Send, breaking `tauri::async_runtime::spawn`. Extract the
	// two scalars needed for logging before moving `FoldersWire` into
	// `WorkspaceDTO` - no clone required.
	let WorkspaceDTO = {
		let Guard = ApplicationState.Workspace.WorkspaceFolders.lock();

		// Cocoon's `WorkspaceNamespace/Index.ts` reads
		// `ExtensionHostInitData.workspace.folders` at shim construction time,
		// then mutates the same array in place on `$deltaWorkspaceFolders`. If
		// `folders` is missing from the init payload, every
		// `vscode.workspace.workspaceFolders` read returns `[]` until a delta
		// fires - which means the git extension boots with zero folders to
		// scan and never calls `createSourceControl`. Emit the folder list
		// inline so extensions that read `workspaceFolders` synchronously in
		// their `activate()` (vscode.git, eamodio.gitlens, typescript) see
		// the real folders.
		let FoldersWire:Vec<Value> = Guard
			.iter()
			.map(|Folder| {
				json!({
					"uri": Folder.URI.to_string(),
					"name": Folder.GetDisplayName(),
					"index": Folder.Index,
				})
			})
			.collect();

		// Extract logging scalars before FoldersWire is moved - avoids clone.
		let FolderCount = FoldersWire.len();

		let FolderSample = FoldersWire.first().map(|F| F.to_string()).unwrap_or_else(|| "<none>".into());

		let IsEmpty = Guard.is_empty();

		drop(Guard); // guard released; no await points follow inside this block

		dev_log!(
			"cocoon",
			"[InitializationData] FoldersWire count={} sample0={}",
			FolderCount,
			FolderSample
		);

		if IsEmpty {
			Value::Null
		} else {
			json!({
				"id": ApplicationState.GetWorkspaceIdentifier()?,
				"name": WorkspaceName,
				"folders": FoldersWire, // moved in - zero extra allocation
				"configuration": ApplicationState.Workspace.WorkspaceConfigurationPath.lock().as_ref().map(|p| p.to_string_lossy()),
				"isUntitled": ApplicationState.Workspace.WorkspaceConfigurationPath.lock().is_none(),
				"transient": false
			})
		}
	};

	let PathResolver = ApplicationHandle.path();

	let AppRoot = PathResolver
		.resource_dir()
		.ok()
		.filter(|P| !P.as_os_str().is_empty() && P.exists())
		.or_else(|| {
			// Tauri's `resource_dir()` returns Err (or an empty/missing
			// path) for raw-binary launches outside the bundle. Probe two
			// fallback layouts so both `.app` and dev launches resolve:
			//
			//   1. `.app/Contents/MacOS/<bin>` → `Contents/Resources/` (shipped bundle,
			//      raw-binary launch from inside the bundle tree).
			//   2. `Element/Mountain/Target/<profile>/<bin>` → `Element/Sky/Target/`
			//      (monorepo dev / raw release).
			let ExeDir = std::env::current_exe()
				.ok()
				.and_then(|P| P.parent().map(|D| D.to_path_buf()))
				.unwrap_or_default();

			let BundleResources = ExeDir.join("../Resources");

			if BundleResources.exists() {
				return Some(BundleResources.canonicalize().unwrap_or(BundleResources));
			}

			let SkyTarget = ExeDir.join("../../../Sky/Target");

			if SkyTarget.exists() {
				return Some(SkyTarget.canonicalize().unwrap_or(SkyTarget));
			}

			None
		})
		.ok_or_else(|| {
			CommonError::ConfigurationLoad {
				Description:"Could not resolve AppRoot from resource_dir, ../Resources, or ../../../Sky/Target"
					.to_string(),
			}
		})?;

	let AppData = PathResolver
		.app_data_dir()
		.map_err(|Error| CommonError::ConfigurationLoad { Description:Error.to_string() })?;

	let LogsLocation = PathResolver
		.app_log_dir()
		.map_err(|Error| CommonError::ConfigurationLoad { Description:Error.to_string() })?;

	let GlobalStorage = AppData.join("User/globalStorage");

	let WorkspaceStorage = AppData.join("User/workspaceStorage");

	// `Url::from_directory_path` fails on relative paths (common in dev
	// launches); propagate as ConfigurationLoad instead of panicking.
	let DirectoryUrl = |Path:&std::path::Path| -> Result<url::Url, CommonError> {
		url::Url::from_directory_path(Path).map_err(|()| {
			CommonError::ConfigurationLoad {
				Description:format!("Failed to build directory URL for {}", Path.display()),
			}
		})
	};

	let AppRootUrl = DirectoryUrl(&AppRoot)?;

	let GlobalStorageUrl = DirectoryUrl(&GlobalStorage)?;

	let WorkspaceStorageUrl = DirectoryUrl(&WorkspaceStorage)?;

	let LogsLocationUrl = DirectoryUrl(&LogsLocation)?;

	Ok(json!({

		// Atom I5: product version + commit + quality come from .env.Land via
		// process env. `Tauri's package_info().version` reads tauri.conf.json
		// which still carries a placeholder "0.0.1" - we can't trust it for
		// extension compat checks. `ProductVersion` from env is the canonical
		// value shared with Wind and Cocoon.
		"commit": std::env::var("ProductCommit").unwrap_or_else(|_| "dev".into()),

		"version": std::env::var("ProductVersion").unwrap_or_else(|_| {
			ApplicationHandle.package_info().version.to_string()
		}),

		"quality": std::env::var("ProductQuality").unwrap_or_else(|_| "development".into()),

		"parentPid": std::process::id(),

		"environment": {

			"isExtensionDevelopmentDebug": false,

			"appName": "Mountain",

			"appHost": "desktop",

			"appUriScheme": "mountain",

			"appLanguage": "en",

			"isExtensionTelemetryLoggingOnly": true,

			"appRoot": AppRootUrl,

			"globalStorageHome": GlobalStorageUrl,

			"workspaceStorageHome": WorkspaceStorageUrl,

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

		"logsLocation": LogsLocationUrl,

		"telemetryInfo": {

			"sessionId": SessionId(),

			"machineId": get_or_generate_machine_id(&AppData).await,

			"firstSessionDate": "2024-01-01T00:00:00.000Z",

			"msftInternal": false
		},

		"extensions": ExtensionsDTO,

		"autoStart": true,

		// UIKind.Desktop
		"uiKind": 1,
	}))
}
