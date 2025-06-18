// @module ApplicationState
// @description Defines the main `ApplicationState` struct, which is the
// central, shared, thread-safe state for the entire Mountain application. It is
// managed by Tauri and accessible to all commands and Environment providers.

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{
		Arc,
		Mutex as StdMutex,
		atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering as AtomicOrdering},
	},
};

use log::{error, info, warn};
use tauri::{ApplicationHandle, Manager, Wry};

use super::{DTO::*, Internal};
use crate::Handler::command::CommandHandler;

// The central, shared, thread-safe state for the entire Mountain application.
// This struct consolidates all dynamic state required by the backend, from
// workspace information and configuration to the state of active User Interface components
// like terminals and webviews.
#[derive(Clone)]
pub struct ApplicationState {
	// --- Workspace State ---
	pub WorkspaceFolders:Arc<StdMutex<Vec<WorkspaceFolderStateDTO>>>,
	pub WorkspaceConfigurationPath:Arc<StdMutex<Option<PathBuf>>>,
	pub IsTrusted:Arc<AtomicBool>,
	pub WindowState:Arc<StdMutex<WindowStateDTO>>,

	// --- Configuration & Storage ---
	pub Configuration:Arc<StdMutex<MergedConfigurationStateDTO>>,
	pub GlobalMemento:Arc<StdMutex<HashMap<String, serde_json::Value>>>,
	pub GlobalMementoPath:PathBuf,
	pub WorkspaceMemento:Arc<StdMutex<HashMap<String, serde_json::Value>>>,
	pub WorkspaceMementoPath:Arc<StdMutex<Option<PathBuf>>>,

	// --- Extension & Provider Management ---
	pub CommandRegistry:Arc<StdMutex<HashMap<String, CommandHandler<Wry>>>>,
	pub LanguageProviders:Arc<StdMutex<HashMap<u32, ProviderRegistrationDTO>>>,
	pub NextProviderHandle:Arc<AtomicU32>,
	pub ScannedExtensions:Arc<StdMutex<HashMap<String, ExtensionDescriptionStateDTO>>>,
	pub EnabledProposedApis:Arc<StdMutex<HashMap<String, Vec<String>>>>,
	pub ExtensionScanPaths:Arc<StdMutex<Vec<PathBuf>>>,

	// --- Feature-specific State ---
	pub DiagnosticsMap:Arc<StdMutex<HashMap<String, HashMap<String, Vec<MarkerDataDTO>>>>>,
	pub OpenDocuments:Arc<StdMutex<HashMap<String, DocumentStateDTO>>>,
	pub OutputChannels:Arc<StdMutex<HashMap<String, OutputChannelStateDTO>>>,
	pub ActiveTerminals:Arc<StdMutex<HashMap<u64, Arc<StdMutex<TerminalStateDTO>>>>>,
	pub NextTerminalIdentifier:Arc<AtomicU64>,
	pub ActiveWebViews:Arc<StdMutex<HashMap<String, WebViewStateDTO>>>,
	pub ActiveCustomDocuments:Arc<StdMutex<HashMap<String, CustomDocumentStateDTO>>>,
	pub ActiveStatusBarItems:Arc<StdMutex<HashMap<String, Common::status_bar::DTO::StatusBarEntryDTO>>>,
	pub ActiveTreeViews:Arc<StdMutex<HashMap<String, TreeViewStateDTO>>>,

	// --- IPC & User Interface State ---
	pub PendingUiRequests: Arc<
		StdMutex<HashMap<String, tokio::sync::oneshot::Sender<Result<serde_json::Value, Common::error::CommonError>>>>,
	>,
}

impl Default for ApplicationState {
	fn default() -> Self {
		info!("[ApplicationState] Initializing default application state...");
		let AppNameForPaths = env!("CARGO_PKG_NAME");
		let AppDataDirectoryPath = dirs::config_dir().map(|p| p.join(AppNameForPaths)).unwrap_or_else(|| {
			warn!(
				"[ApplicationState] Could not get config dir. Using relative path '.{}-appdata'.",
				AppNameForPaths
			);
			PathBuf::from(format!(".{}-appdata", AppNameForPaths))
		});

		// This must be synchronous because the async runtime isn't available in
		// `default`. A proper async version is used in the handler logic.
		if !AppDataDirectoryPath.exists() {
			if let Err(e) = std::fs::create_dir_all(&AppDataDirectoryPath) {
				error!(
					"[ApplicationState] CRITICAL: Failed to create app data directory at '{}': {}.",
					AppDataDirectoryPath.display(),
					e
				);
			}
		}

		let GlobalMementoFilePath = Internal::ResolveMementoStorageFilePath(&AppDataDirectoryPath, true, "");
		let InitialGlobalMementoMap = Internal::LoadInitialMementoFromDisk(&GlobalMementoFilePath);
		let InitialCommandRegistryMap = crate::Handler::command::RegisterNativeCommand();

		info!("[ApplicationState] Default state initialization complete.");
		Self {
			WorkspaceFolders:Arc::new(StdMutex::new(Vec::new())),
			WorkspaceConfigurationPath:Arc::new(StdMutex::new(None)),
			IsTrusted:Arc::new(AtomicBool::new(false)),
			WindowState:Arc::new(StdMutex::new(Default::default())),
			Configuration:Arc::new(StdMutex::new(MergedConfigurationStateDTO::default())),
			GlobalMemento:Arc::new(StdMutex::new(InitialGlobalMementoMap)),
			GlobalMementoPath:GlobalMementoFilePath,
			WorkspaceMemento:Arc::new(StdMutex::new(HashMap::new())),
			WorkspaceMementoPath:Arc::new(StdMutex::new(None)),
			CommandRegistry:Arc::new(StdMutex::new(InitialCommandRegistryMap)),
			DiagnosticsMap:Arc::new(StdMutex::new(HashMap::new())),
			OpenDocuments:Arc::new(StdMutex::new(HashMap::new())),
			OutputChannels:Arc::new(StdMutex::new(HashMap::new())),
			LanguageProviders:Arc::new(StdMutex::new(HashMap::new())),
			NextProviderHandle:Arc::new(AtomicU32::new(1)),
			ScannedExtensions:Arc::new(StdMutex::new(HashMap::new())),
			EnabledProposedApis:Arc::new(StdMutex::new(HashMap::new())),
			ExtensionScanPaths:Arc::new(StdMutex::new(Vec::new())),
			ActiveTerminals:Arc::new(StdMutex::new(HashMap::new())),
			NextTerminalIdentifier:Arc::new(AtomicU64::new(1)),
			PendingUiRequests:Arc::new(StdMutex::new(HashMap::new())),
			ActiveWebViews:Arc::new(StdMutex::new(HashMap::new())),
			ActiveCustomDocuments:Arc::new(StdMutex::new(HashMap::new())),
			ActiveStatusBarItems:Arc::new(StdMutex::new(HashMap::new())),
			ActiveTreeViews:Arc::new(StdMutex::new(HashMap::new())),
		}
	}
}

impl ApplicationState {
	// Generates a unique, filesystem-safe identifier string for the current
	// workspace.
	pub fn GetWorkspaceIdentifier(&self) -> Result<String, String> {
		let config_path_guard = self
			.WorkspaceConfigurationPath
			.lock()
			.map_err(|e| format!("[AppState] Lock error on WorkspaceConfigurationPath: {}", e))?;
		if let Some(config_path) = config_path_guard.as_ref() {
			return Ok(config_path.file_name().unwrap_or_default().to_string_lossy().into_owned());
		}
		drop(config_path_guard);

		let folders_guard = self
			.WorkspaceFolders
			.lock()
			.map_err(|e| format!("[AppState] Lock error on WorkspaceFolders: {}", e))?;
		if let Some(first_folder) = folders_guard.first() {
			let path_str = first_folder.Uri.path();
			// Create a more stable hash for the identifier.
			let hash = format!("{:x}", md5::compute(path_str));
			return Ok(format!(
				"{}-{}",
				first_folder.Name.replace(|c:char| !c.is_alphanumeric(), "_"),
				&hash[..8]
			));
		}

		Ok("NO_WORKSPACE".to_string())
	}

	// Gets the display name for the current workspace.
	pub fn GetWorkspaceName(&self) -> Result<String, String> {
		let config_path_guard = self
			.WorkspaceConfigurationPath
			.lock()
			.map_err(|e| format!("[AppState] Lock error on WorkspaceConfigurationPath: {}", e))?;
		if let Some(stem) = config_path_guard.as_ref().and_then(|p| p.file_stem()) {
			return Ok(stem.to_string_lossy().into_owned());
		}
		drop(config_path_guard);

		let folders_guard = self
			.WorkspaceFolders
			.lock()
			.map_err(|e| format!("[AppState] Lock error on WorkspaceFolders: {}", e))?;
		if let Some(first_folder) = folders_guard.first() {
			return Ok(first_folder.Name.clone());
		}

		Ok("Untitled Workspace".to_string())
	}

	// Returns the next available unique identifier for a language provider
	// registration.
	pub fn GetNextProviderHandle(&self) -> u32 { self.NextProviderHandle.fetch_add(1, AtomicOrdering::Relaxed) }

	// Returns the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.NextTerminalIdentifier.fetch_add(1, AtomicOrdering::Relaxed) }

	// Asynchronously scans configured paths for extensions and populates the
	// state.
	pub async fn ScanExtensions(&self, app_handle:&ApplicationHandle<Wry>) {
		crate::Handler::extension_management::ScanExtensionsAndPopulateState(app_handle, self).await;
	}

	// Updates the path to the workspace memento file and reloads its content
	// from disk.
	pub fn UpdateWorkspaceMementoPathAndReload(&self, app_data_directory:&Path) -> Result<(), String> {
		let workspace_identifier = self.GetWorkspaceIdentifier()?;
		let mut path_guard = self
			.WorkspaceMementoPath
			.lock()
			.map_err(|e| format!("[AppState] Lock error on WorkspaceMementoPath: {}", e))?;

		if workspace_identifier == "NO_WORKSPACE" {
			if path_guard.is_some() {
				*path_guard = None;
				self.WorkspaceMemento
					.lock()
					.map_err(|e| format!("[AppState] Lock error on WorkspaceMemento: {}", e))?
					.clear();
			}
			return Ok(());
		}

		let new_memento_path =
			Internal::ResolveMementoStorageFilePath(app_data_directory, false, &workspace_identifier);
		if path_guard.as_ref() != Some(&new_memento_path) {
			if let Some(parent) = new_memento_path.parent() {
				if !parent.exists() {
					std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
				}
			}
			*path_guard = Some(new_memento_path.clone());
			let new_memento_content = Internal::LoadInitialMementoFromDisk(&new_memento_path);
			*self
				.WorkspaceMemento
				.lock()
				.map_err(|e| format!("[AppState] Lock error on WorkspaceMemento: {}", e))? = new_memento_content;
		}
		Ok(())
	}
}
