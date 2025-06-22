// File: Mountain/Source/ApplicationState/ApplicationState.rs
// Role: Defines the main `ApplicationState` struct, which is the central,
// shared,       thread-safe state container for the entire Mountain
// application. Responsibilities:
//   - Hold all runtime state for services like configuration, extensions,
//     documents, and UI.
//   - Provide thread-safe access to this state via `Arc<Mutex<...>>`.
//   - Be managed by Tauri and accessible to all command handlers and
//     Environment providers.

//! # ApplicationState Struct
//!
//! Defines the main `ApplicationState` struct, which is the central, shared,
//! thread-safe state container for the entire Mountain application. It is
//! managed by Tauri and is accessible to all command handlers and Environment
//! providers.

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{
		Arc,
		Mutex as StandardMutex,
		PoisonError,
		atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering as AtomicOrdering},
	},
};

use Common::{
	Error::CommonError::CommonError,
	SourceControlManagement::DTO::{
		SourceControlManagementGroupDTO::SourceControlManagementGroupDTO,
		SourceControlManagementProviderDTO::SourceControlManagementProviderDTO,
		SourceControlManagementResourceDTO::SourceControlManagementResourceDTO,
	},
	StatusBar::DTO::StatusBarEntryDTO::StatusBarEntryDTO,
};
use log::{error, info, warn};
use tauri::Wry;

use super::{
	DTO::{
		CustomDocumentStateDTO::CustomDocumentStateDTO,
		DocumentStateDTO::DocumentStateDTO,
		ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
		MarkerDataDTO::MarkerDataDTO,
		MergedConfigurationStateDTO::MergedConfigurationStateDTO,
		OutputChannelStateDTO::OutputChannelStateDTO,
		ProviderRegistrationDTO::ProviderRegistrationDTO,
		TerminalStateDTO::TerminalStateDTO,
		TreeViewStateDTO::TreeViewStateDTO,
		WebViewStateDTO::WebViewStateDTO,
		WindowStateDTO::WindowStateDTO,
		WorkSpaceFolderStateDTO::WorkSpaceFolderStateDTO,
	},
	Internal,
};
use crate::Environment::CommandProvider::CommandHandler;

/// The central, shared, thread-safe state for the entire Mountain application.
#[derive(Clone)]
pub struct ApplicationState {
	// --- WorkSpace State ---
	pub WorkSpaceFolders:Arc<StandardMutex<Vec<WorkSpaceFolderStateDTO>>>,
	pub WorkSpaceConfigurationPath:Arc<StandardMutex<Option<PathBuf>>>,
	pub IsTrusted:Arc<AtomicBool>,
	pub WindowState:Arc<StandardMutex<WindowStateDTO>>,
	pub ActiveDocumentURI:Arc<StandardMutex<Option<String>>>,

	// --- Configuration & Storage ---
	pub Configuration:Arc<StandardMutex<MergedConfigurationStateDTO>>,
	pub GlobalMemento:Arc<StandardMutex<HashMap<String, serde_json::Value>>>,
	pub GlobalMementoPath:PathBuf,
	pub WorkSpaceMemento:Arc<StandardMutex<HashMap<String, serde_json::Value>>>,
	pub WorkSpaceMementoPath:Arc<StandardMutex<Option<PathBuf>>>,

	// --- Extension & Provider Management ---
	pub CommandRegistry:Arc<StandardMutex<HashMap<String, CommandHandler<Wry>>>>,
	pub LanguageProviders:Arc<StandardMutex<HashMap<u32, ProviderRegistrationDTO>>>,
	pub NextProviderHandle:Arc<AtomicU32>,
	pub ScannedExtensions:Arc<StandardMutex<HashMap<String, ExtensionDescriptionStateDTO>>>,
	pub EnabledProposedAPIs:Arc<StandardMutex<HashMap<String, Vec<String>>>>,
	pub ExtensionScanPaths:Arc<StandardMutex<Vec<PathBuf>>>,

	// --- Feature-specific State ---
	pub DiagnosticsMap:Arc<StandardMutex<HashMap<String, HashMap<String, Vec<MarkerDataDTO>>>>>,
	pub OpenDocuments:Arc<StandardMutex<HashMap<String, DocumentStateDTO>>>,
	pub OutputChannels:Arc<StandardMutex<HashMap<String, OutputChannelStateDTO>>>,
	pub ActiveTerminals:Arc<StandardMutex<HashMap<u64, Arc<StandardMutex<TerminalStateDTO>>>>>,
	pub NextTerminalIdentifier:Arc<AtomicU64>,
	pub ActiveWebViews:Arc<StandardMutex<HashMap<String, WebViewStateDTO>>>,
	pub ActiveCustomDocuments:Arc<StandardMutex<HashMap<String, CustomDocumentStateDTO>>>,
	pub ActiveStatusBarItems:Arc<StandardMutex<HashMap<String, StatusBarEntryDTO>>>,
	pub ActiveTreeViews:Arc<StandardMutex<HashMap<String, TreeViewStateDTO>>>,
	pub ScmProviders:Arc<StandardMutex<HashMap<u32, SourceControlManagementProviderDTO>>>,
	pub ScmGroups:Arc<StandardMutex<HashMap<u32, HashMap<String, SourceControlManagementGroupDTO>>>>,
	pub ScmResources:Arc<StandardMutex<HashMap<u32, HashMap<String, Vec<SourceControlManagementResourceDTO>>>>>,
	pub NextScmProviderHandle:Arc<AtomicU32>,

	// --- IPC & User Interface State ---
	pub PendingUserInterfaceRequests:
		Arc<StandardMutex<HashMap<String, tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>>>>,
}

fn map_lock_error<T>(e:PoisonError<T>) -> CommonError { CommonError::StateLockPoisoned { Context:e.to_string() } }

impl Default for ApplicationState {
	fn default() -> Self {
		info!("[ApplicationState] Initializing default application state...");
		let ApplicationNameForPaths = env!("CARGO_PKG_NAME");
		let ApplicationDataDirectoryPath =
			dirs::config_dir().map(|p| p.join(ApplicationNameForPaths)).unwrap_or_else(|| {
				warn!(
					"[ApplicationState] Could not get config dir. Using relative path '.{}-appdata'.",
					ApplicationNameForPaths
				);
				PathBuf::from(format!(".{}-appdata", ApplicationNameForPaths))
			});

		// This must be synchronous because the async runtime isn't available yet.
		if !ApplicationDataDirectoryPath.exists() {
			if let Err(e) = std::fs::create_dir_all(&ApplicationDataDirectoryPath) {
				error!(
					"[ApplicationState] CRITICAL: Failed to create app data directory at '{}': {}.",
					ApplicationDataDirectoryPath.display(),
					e
				);
			}
		}

		let GlobalMementoFilePath = Internal::ResolveMementoStorageFilePath(&ApplicationDataDirectoryPath, true, "");
		let InitialGlobalMementoMap = Internal::LoadInitialMementoFromDisk(&GlobalMementoFilePath);

		info!("[ApplicationState] Default state initialization complete.");
		Self {
			WorkSpaceFolders:Arc::new(StandardMutex::new(Vec::new())),
			WorkSpaceConfigurationPath:Arc::new(StandardMutex::new(None)),
			IsTrusted:Arc::new(AtomicBool::new(false)),
			WindowState:Arc::new(StandardMutex::new(Default::default())),
			ActiveDocumentURI:Arc::new(StandardMutex::new(None)),
			Configuration:Arc::new(StandardMutex::new(MergedConfigurationStateDTO::default())),
			GlobalMemento:Arc::new(StandardMutex::new(InitialGlobalMementoMap)),
			GlobalMementoPath:GlobalMementoFilePath,
			WorkSpaceMemento:Arc::new(StandardMutex::new(HashMap::new())),
			WorkSpaceMementoPath:Arc::new(StandardMutex::new(None)),
			CommandRegistry:Arc::new(StandardMutex::new(HashMap::new())),
			LanguageProviders:Arc::new(StandardMutex::new(HashMap::new())),
			NextProviderHandle:Arc::new(AtomicU32::new(1)),
			ScannedExtensions:Arc::new(StandardMutex::new(HashMap::new())),
			EnabledProposedAPIs:Arc::new(StandardMutex::new(HashMap::new())),
			ExtensionScanPaths:Arc::new(StandardMutex::new(Vec::new())),
			DiagnosticsMap:Arc::new(StandardMutex::new(HashMap::new())),
			OpenDocuments:Arc::new(StandardMutex::new(HashMap::new())),
			OutputChannels:Arc::new(StandardMutex::new(HashMap::new())),
			ActiveTerminals:Arc::new(StandardMutex::new(HashMap::new())),
			NextTerminalIdentifier:Arc::new(AtomicU64::new(1)),
			ActiveWebViews:Arc::new(StandardMutex::new(HashMap::new())),
			ActiveCustomDocuments:Arc::new(StandardMutex::new(HashMap::new())),
			ActiveStatusBarItems:Arc::new(StandardMutex::new(HashMap::new())),
			ActiveTreeViews:Arc::new(StandardMutex::new(HashMap::new())),
			ScmProviders:Arc::new(StandardMutex::new(HashMap::new())),
			ScmGroups:Arc::new(StandardMutex::new(HashMap::new())),
			ScmResources:Arc::new(StandardMutex::new(HashMap::new())),
			NextScmProviderHandle:Arc::new(AtomicU32::new(1)),
			PendingUserInterfaceRequests:Arc::new(StandardMutex::new(HashMap::new())),
		}
	}
}

impl ApplicationState {
	/// Generates a unique, filesystem-safe identifier for the current
	/// workspace.
	pub fn GetWorkSpaceIdentifier(&self) -> Result<String, CommonError> {
		let ConfigurationPathGuard = self.WorkSpaceConfigurationPath.lock().map_err(map_lock_error)?;
		if let Some(ConfigurationPath) = ConfigurationPathGuard.as_ref() {
			return Ok(ConfigurationPath.file_name().unwrap_or_default().to_string_lossy().into_owned());
		}
		drop(ConfigurationPathGuard);

		let FoldersGuard = self.WorkSpaceFolders.lock().map_err(map_lock_error)?;
		if let Some(FirstFolder) = FoldersGuard.first() {
			let PathString = FirstFolder.URI.path();
			// Create a more stable hash for the identifier.
			let Hash = format!("{:x}", md5::compute(PathString));
			return Ok(format!(
				"{}-{}",
				FirstFolder.Name.replace(|c:char| !c.is_alphanumeric(), "_"),
				&Hash[..8]
			));
		}

		Ok("NO_WORKSPACE".to_string())
	}

	/// Returns the next available unique identifier for a language provider
	/// registration.
	pub fn GetNextProviderHandle(&self) -> u32 { self.NextProviderHandle.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Returns the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.NextTerminalIdentifier.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Returns the next available unique identifier for an SCM provider
	/// instance.
	pub fn GetNextScmProviderHandle(&self) -> u32 { self.NextScmProviderHandle.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Updates the path to the workspace memento file and reloads its content
	/// from disk.
	pub fn UpdateWorkSpaceMementoPathAndReload(&self, ApplicationDataDirectory:&Path) -> Result<(), String> {
		let WorkSpaceIdentifier = self.GetWorkSpaceIdentifier().map_err(|e| e.to_string())?;
		let mut PathGuard = self.WorkSpaceMementoPath.lock().map_err(|e| e.to_string())?;

		if WorkSpaceIdentifier == "NO_WORKSPACE" {
			if PathGuard.is_some() {
				*PathGuard = None;
				self.WorkSpaceMemento.lock().map_err(|e| e.to_string())?.clear();
			}
			return Ok(());
		}

		let NewMementoPath =
			Internal::ResolveMementoStorageFilePath(ApplicationDataDirectory, false, &WorkSpaceIdentifier);
		if PathGuard.as_ref() != Some(&NewMementoPath) {
			if let Some(Parent) = NewMementoPath.parent() {
				if !Parent.exists() {
					std::fs::create_dir_all(Parent).map_err(|e| e.to_string())?;
				}
			}
			*PathGuard = Some(NewMementoPath.clone());
			let NewMementoContent = Internal::LoadInitialMementoFromDisk(&NewMementoPath);
			*self.WorkSpaceMemento.lock().map_err(|e| e.to_string())? = NewMementoContent;
		}
		Ok(())
	}
}
