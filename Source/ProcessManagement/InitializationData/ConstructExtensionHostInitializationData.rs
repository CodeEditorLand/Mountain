//! `InitializationData::ConstructExtensionHostInitializationData`

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

/// Constructs the `IExtensionHostInitData` payload sent to `Cocoon`.
pub async fn Fn(Environment:&MountainEnvironment) -> Result<Value, CommonError> {
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
		let Guard = ApplicationState.Workspace.WorkspaceFolders.lock().unwrap();

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
				"configuration": ApplicationState.Workspace.WorkspaceConfigurationPath.lock().unwrap().as_ref().map(|p| p.to_string_lossy()),
				"isUntitled": ApplicationState.Workspace.WorkspaceConfigurationPath.lock().unwrap().is_none(),
				"transient": false
			})
		}
	};

	let PathResolver = ApplicationHandle.Path();

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

			"sessionId": SessionId(),

			"machineId": GetOrGenerateMachineId(&AppData).await,

			"firstSessionDate": "2024-01-01T00:00:00.000Z",

			"msftInternal": false
		},

		"extensions": ExtensionsDTO,

		"autoStart": true,

		// UIKind.Desktop
		"uiKind": 1,
	}))
}
