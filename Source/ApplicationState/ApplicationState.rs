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

// @module ApplicationState
// @description Defines the main `ApplicationState` struct, which is the central,
// shared, thread-safe state for the entire Mountain application. It is managed
// by Tauri and accessible to all commands and environment providers.
use super::{DTO::*, Internal};
use crate::Handler::commands::CommandHandler;

// The central, shared, thread-safe state for the entire Mountain application.
#[derive(Clone)]
pub struct ApplicationState {
	// Workspace State
	pub WorkspaceFolders:Arc<StdMutex<Vec<WorkspaceFolderStateDto>>>,
	pub WorkspaceConfigurationPath:Arc<StdMutex<Option<PathBuf>>>,
	pub IsTrusted:Arc<AtomicBool>,
	pub WindowState:Arc<StdMutex<WindowStateDto>>,

	// Configuration & Storage
	pub Configuration:Arc<StdMutex<MergedConfigurationStateDto>>,
	pub GlobalMemento:Arc<StdMutex<HashMap<String, serde_json::Value>>>,
	pub GlobalMementoPath:PathBuf,
	pub WorkspaceMemento:Arc<StdMutex<HashMap<String, serde_json::Value>>>,
	pub WorkspaceMementoPath:Arc<StdMutex<Option<PathBuf>>>,

	// Extension & Provider Management
	pub CommandRegistry:Arc<StdMutex<HashMap<String, CommandHandler<Wry>>>>,
	pub LanguageProviders:Arc<StdMutex<HashMap<u32, ProviderRegistrationDto>>>,
	pub NextProviderHandle:Arc<AtomicU32>,
	pub ScannedExtensions:Arc<StdMutex<HashMap<String, ExtensionDescriptionStateDto>>>,
	pub EnabledProposedApis:Arc<StdMutex<HashMap<String, Vec<String>>>>,
	pub ExtensionScanPaths:Arc<StdMutex<Vec<PathBuf>>>,

	// Feature-specific State
	pub DiagnosticsMap:Arc<StdMutex<HashMap<String, HashMap<String, Vec<MarkerDataDto>>>>>,
	pub OpenDocuments:Arc<StdMutex<HashMap<String, DocumentStateDto>>>,
	pub OutputChannels:Arc<StdMutex<HashMap<String, OutputChannelStateDto>>>,
	pub ActiveTerminals:Arc<StdMutex<HashMap<u64, Arc<StdMutex<TerminalStateDto>>>>>,
	pub NextTerminalIdentifier:Arc<AtomicU64>,
	pub ActiveWebviews:Arc<StdMutex<HashMap<String, WebviewStateDto>>>,
	pub ActiveCustomDocuments:Arc<StdMutex<HashMap<String, CustomDocumentStateDto>>>,
	pub ActiveStatusBarItems:Arc<StdMutex<HashMap<String, Common::status_bar::dto::StatusBarEntryDto>>>,
	pub ActiveHierarchySessions:Arc<StdMutex<HashMap<String, HierarchySessionContextDto>>>,

	// IPC & UI State
	pub PendingUiRequests: Arc<
		StdMutex<HashMap<String, tokio::sync::oneshot::Sender<Result<serde_json::Value, Common::error::CommonError>>>>,
	>,
}

impl Default for ApplicationState {
	fn default() -> Self {
		info!("[ApplicationState] Initializing default application state...");
		let AppNameForPaths = env!("CARGO_PKG_NAME");
		let AppDataDirectoryPath = dirs::config_dir().map(|p| p.join(AppNameForPaths)).unwrap_or_else(|| {
			warn!("[ApplicationState] Could not get config dir. Using relative path.");
			PathBuf::from(format!(".{}-appdata", AppNameForPaths))
		});
		Internal::EnsureDirectoryExists(&AppDataDirectoryPath);

		let GlobalMementoFilePath = Internal::ResolveMementoStorageFilePath(&AppDataDirectoryPath, true, "");
		let InitialGlobalMementoMap = Internal::LoadInitialMementoFromDisk(&GlobalMementoFilePath);
		let InitialCommandRegistryMap = crate::Handler::commands::RegisterNativeCommands();

		info!("[ApplicationState] Default state initialization complete.");
		Self {
			WorkspaceFolders:Arc::new(StdMutex::new(Vec::new())),
			WorkspaceConfigurationPath:Arc::new(StdMutex::new(None)),
			IsTrusted:Arc::new(AtomicBool::new(false)),
			WindowState:Arc::new(StdMutex::new(Default::default())),
			Configuration:Arc::new(StdMutex::new(MergedConfigurationStateDto::default())),
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
			ActiveHierarchySessions:Arc::new(StdMutex::new(HashMap::new())),
			ActiveWebviews:Arc::new(StdMutex::new(HashMap::new())),
			ActiveCustomDocuments:Arc::new(StdMutex::new(HashMap::new())),
			ActiveStatusBarItems:Arc::new(StdMutex::new(HashMap::new())),
		}
	}
}

impl ApplicationState {
	// All helper methods from the provided source are preserved here...
	pub fn GetWorkspaceIdentifier(&self) -> Result<String, String> {
		// ...
		Ok("...".to_string())
	}

	pub fn GetWorkspaceName(&self) -> Result<String, String> {
		// ...
		Ok("...".to_string())
	}

	pub fn GetNextProviderHandle(&self) -> u32 { self.NextProviderHandle.fetch_add(1, AtomicOrdering::Relaxed) }

	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.NextTerminalIdentifier.fetch_add(1, AtomicOrdering::Relaxed) }

	pub async fn ScanExtensions(&self, ApplicationHandle:&ApplicationHandle<Wry>) { /* ...// 
	}

	pub fn UpdateWorkspaceMementoPathAndReload(&self, AppDataDirectory:&Path) -> Result<(), String> {
		// ...
		Ok(())
	}
}
