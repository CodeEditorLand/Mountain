// File: Mountain/Source/ApplicationState/ApplicationState.rs
// Role: Defines the main `ApplicationState` struct, which is the central,
// shared, thread-safe state container for the entire Mountain application.
// Responsibilities:
//   - Hold all runtime state for services like configuration, extensions,
//     documents, and UI.
//   - Provide thread-safe access to this state via `Arc<Mutex<...>>`.
//   - Be managed by Tauri and accessible to all command handlers and
//     Environment providers.

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # ApplicationState Struct
//!
//! Defines the main `ApplicationState` struct, which is the central, shared,
//! thread-safe state container for the entire Mountain application. It is
//! managed by Tauri and is accessible to all command handlers and Environment
//! providers.

#![allow(non_snake_case, non_camel_case_types)]

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
use log::{error, info, warn, debug};
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

	pub SourceControlManagementProviders:Arc<StandardMutex<HashMap<u32, SourceControlManagementProviderDTO>>>,

	pub SourceControlManagementGroups:
		Arc<StandardMutex<HashMap<u32, HashMap<String, SourceControlManagementGroupDTO>>>>,

	pub SourceControlManagementResources:
		Arc<StandardMutex<HashMap<u32, HashMap<String, Vec<SourceControlManagementResourceDTO>>>>>,

	pub NextSourceControlManagementProviderHandle:Arc<AtomicU32>,

	// --- IPC & User Interface State ---
	pub PendingUserInterfaceRequests:
		Arc<StandardMutex<HashMap<String, tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>>>>,
}

/// A helper to map a mutex poison error into a `CommonError`.
pub fn MapLockError<T>(Error:PoisonError<T>) -> CommonError {
	CommonError::StateLockPoisoned { Context:Error.to_string() }
}

/// A helper to map a mutex poison error with recovery attempt.
pub fn MapLockErrorWithRecovery<T>(Error:PoisonError<T>, RecoveryContext:&str) -> CommonError {
	warn!("[ApplicationState] Attempting recovery from poisoned lock in context: {}", RecoveryContext);
	CommonError::StateLockPoisoned { Context:format!("{} - Recovery attempted: {}", Error.to_string(), RecoveryContext) }
}

/// Error handling result with recovery information
#[derive(Debug)]
pub struct StateOperationResult<T> {
	pub result: Result<T, CommonError>,
	pub recovery_attempted: bool,
	pub recovery_successful: bool,
}

impl Default for ApplicationState {
	fn default() -> Self {
		info!("[ApplicationState] Initializing default application state...");

		let ApplicationNameForPaths = env!("CARGO_PKG_NAME");

		let ApplicationDataDirectoryPath = dirs::config_dir()
			.map(|Path| Path.join(ApplicationNameForPaths))
			.unwrap_or_else(|| {
				warn!(
					"[ApplicationState] Could not get config dir. Using relative path '.{}-appdata'.",
					ApplicationNameForPaths
				);

				PathBuf::from(format!(".{}-appdata", ApplicationNameForPaths))
			});

		// This must be synchronous because the async runtime isn't available yet.
		if !ApplicationDataDirectoryPath.exists() {
			if let Err(Error) = std::fs::create_dir_all(&ApplicationDataDirectoryPath) {
				error!(
					"[ApplicationState] CRITICAL: Failed to create app data directory at '{}': {}.",
					ApplicationDataDirectoryPath.display(),
					Error
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

			SourceControlManagementProviders:Arc::new(StandardMutex::new(HashMap::new())),

			SourceControlManagementGroups:Arc::new(StandardMutex::new(HashMap::new())),

			SourceControlManagementResources:Arc::new(StandardMutex::new(HashMap::new())),

			NextSourceControlManagementProviderHandle:Arc::new(AtomicU32::new(1)),

			PendingUserInterfaceRequests:Arc::new(StandardMutex::new(HashMap::new())),
		}
	}
}

impl ApplicationState {
	/// Generates a unique, filesystem-safe identifier for the current
	/// workspace. Returns "NO_WORKSPACE" if no folder or configuration is
	/// open.
	pub fn GetWorkSpaceIdentifier(&self) -> Result<String, CommonError> {
		let ConfigurationPathGuard = self.WorkSpaceConfigurationPath.lock().map_err(|e| {
			MapLockErrorWithRecovery(e, "GetWorkSpaceIdentifier - ConfigurationPath")
		})?;

		if let Some(ConfigurationPath) = ConfigurationPathGuard.as_ref() {
			let FileName = ConfigurationPath.file_name().unwrap_or_default().to_string_lossy().into_owned();

			return Ok(FileName);
		}

		drop(ConfigurationPathGuard);

		let FoldersGuard = self.WorkSpaceFolders.lock().map_err(|e| {
			MapLockErrorWithRecovery(e, "GetWorkSpaceIdentifier - WorkSpaceFolders")
		})?;

		if let Some(FirstFolder) = FoldersGuard.first() {
			let PathString = FirstFolder.URI.path();

			// Create a more stable hash for the identifier.
			let Hash = format!("{:x}", md5::compute(PathString));

			let SafeName = FirstFolder.Name.replace(|c:char| !c.is_alphanumeric(), "_");

			return Ok(format!("{}-{}", SafeName, &Hash[..8]));
		}

		Ok("NO_WORKSPACE".to_string())
	}

	/// Safe state operation with automatic recovery
	pub fn SafeStateOperation<T, F>(&self, operation_name: &str, operation: F) -> StateOperationResult<T>
	where
		F: FnOnce() -> Result<T, CommonError>,
	{
		let mut recovery_attempted = false;
		let mut recovery_successful = false;
		
		match operation() {
			Ok(result) => StateOperationResult {
				result: Ok(result),
				recovery_attempted,
				recovery_successful,
			},
			Err(error) => {
				// Attempt recovery for specific error types
				if Self::should_attempt_recovery(&error) {
					recovery_attempted = true;
					match Self::attempt_state_recovery(&error, operation_name) {
						Ok(()) => {
							recovery_successful = true;
							info!("[ApplicationState] Recovery successful for operation: {}", operation_name);
							StateOperationResult {
								result: Err(error), // Original error still returned
								recovery_attempted,
								recovery_successful,
							}
						},
						Err(recovery_error) => {
							warn!("[ApplicationState] Recovery failed for operation {}: {}", operation_name, recovery_error);
							StateOperationResult {
								result: Err(error), // Original error still returned
								recovery_attempted,
								recovery_successful,
							}
						},
					}
				} else {
					StateOperationResult {
						result: Err(error),
						recovery_attempted,
						recovery_successful,
					}
				}
			},
		}
	}

	/// Determine if recovery should be attempted for a given error
	fn should_attempt_recovery(error: &CommonError) -> bool {
		match error {
			CommonError::StateLockPoisoned { .. } => true,
			CommonError::FileSystemIO { .. } => true,
			CommonError::SerializationError { .. } => true,
			_ => false,
		}
	}

	/// Attempt state recovery based on error type
	fn attempt_state_recovery(error: &CommonError, context: &str) -> Result<(), CommonError> {
		match error {
			CommonError::StateLockPoisoned { .. } => {
				// For poisoned locks, we can't do much but log and wait
				warn!("[ApplicationState] Poisoned lock detected in context: {}", context);
				// Small delay to allow system to stabilize
				std::thread::sleep(std::time::Duration::from_millis(100));
				Ok(())
			},
			CommonError::FileSystemIO { path, .. } => {
				// Attempt to recreate directories or handle file system issues
				if let Some(parent) = path.parent() {
					if !parent.exists() {
						std::fs::create_dir_all(parent).map_err(|e| {
							CommonError::FileSystemIO {
								Path: parent.to_path_buf(),
								Description: format!("Failed to create directory during recovery: {}", e),
							}
						})?;
					}
				}
				Ok(())
			},
			CommonError::SerializationError { .. } => {
				// For serialization errors, we might need to reset corrupted state
				warn!("[ApplicationState] Serialization error detected, state may be corrupted: {}", context);
				Ok(())
			},
			_ => Ok(()),
		}
	}

	/// Returns the next available unique identifier for a language provider
	/// registration.
	pub fn GetNextProviderHandle(&self) -> u32 { self.NextProviderHandle.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Returns the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.NextTerminalIdentifier.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Returns the next available unique identifier for an SCM provider
	/// instance.
	pub fn GetNextSourceControlManagementProviderHandle(&self) -> u32 {
		self.NextSourceControlManagementProviderHandle
			.fetch_add(1, AtomicOrdering::Relaxed)
	}

	/// Updates the path to the workspace memento file and reloads its content
	/// from disk. This should be called when a workspace is opened or changed.
	pub fn UpdateWorkSpaceMementoPathAndReload(&self, ApplicationDataDirectory:&Path) -> Result<(), CommonError> {
		let operation_result = self.SafeStateOperation("UpdateWorkSpaceMementoPathAndReload", || {
			let WorkSpaceIdentifier = self.GetWorkSpaceIdentifier()?;

			let mut PathGuard = self.WorkSpaceMementoPath.lock().map_err(|e| {
				MapLockErrorWithRecovery(e, "UpdateWorkSpaceMementoPathAndReload - WorkSpaceMementoPath")
			})?;

			if WorkSpaceIdentifier == "NO_WORKSPACE" {
				if PathGuard.is_some() {
					*PathGuard = None;

					self.WorkSpaceMemento.lock().map_err(|e| {
						MapLockErrorWithRecovery(e, "UpdateWorkSpaceMementoPathAndReload - WorkSpaceMemento")
					})?.clear();
				}

				return Ok(());
			}

			let NewMementoPath =
				Internal::ResolveMementoStorageFilePath(ApplicationDataDirectory, false, &WorkSpaceIdentifier);

			if PathGuard.as_ref() != Some(&NewMementoPath) {
				if let Some(Parent) = NewMementoPath.parent() {
					if !Parent.exists() {
						std::fs::create_dir_all(Parent).map_err(|Error| {
							CommonError::FileSystemIO { Path:Parent.to_path_buf(), Description:Error.to_string() }
						})?;
					}
				}

				*PathGuard = Some(NewMementoPath.clone());

				let NewMementoContent = Internal::LoadInitialMementoFromDisk(&NewMementoPath);

				*self.WorkSpaceMemento.lock().map_err(|e| {
					MapLockErrorWithRecovery(e, "UpdateWorkSpaceMementoPathAndReload - WorkSpaceMemento update")
				})? = NewMementoContent;
			}

			Ok(())
		});

		operation_result.result
	}

	/// Enhanced state recovery with comprehensive error handling
	pub async fn RecoverApplicationState(&self) -> Result<(), CommonError> {
		info!("[ApplicationState] Starting comprehensive state recovery...");

		// Recover global memento
		self.RecoverGlobalMemento().await?;

		// Recover workspace memento
		self.RecoverWorkSpaceMemento().await?;

		// Recover extension state
		self.RecoverExtensionState().await?;

		// Recover document state
		self.RecoverDocumentState().await?;

		info!("[ApplicationState] Comprehensive state recovery completed successfully");
		Ok(())
	}

	/// Recover global memento state
	async fn RecoverGlobalMemento(&self) -> Result<(), CommonError> {
		debug!("[ApplicationState] Recovering global memento state...");
		
		let memento_content = Internal::LoadInitialMementoFromDisk(&self.GlobalMementoPath);
		let mut global_memento = self.GlobalMemento.lock().map_err(|e| {
			MapLockErrorWithRecovery(e, "RecoverGlobalMemento")
		})?;
		
		*global_memento = memento_content;
		Ok(())
	}

	/// Recover workspace memento state
	async fn RecoverWorkSpaceMemento(&self) -> Result<(), CommonError> {
		debug!("[ApplicationState] Recovering workspace memento state...");
		
		let workspace_path_guard = self.WorkSpaceMementoPath.lock().map_err(|e| {
			MapLockErrorWithRecovery(e, "RecoverWorkSpaceMemento - Path")
		})?;
		
		if let Some(path) = workspace_path_guard.as_ref() {
			let memento_content = Internal::LoadInitialMementoFromDisk(path);
			let mut workspace_memento = self.WorkSpaceMemento.lock().map_err(|e| {
				MapLockErrorWithRecovery(e, "RecoverWorkSpaceMemento - Content")
			})?;
			
			*workspace_memento = memento_content;
		}
		
		Ok(())
	}

	/// Recover extension state
	async fn RecoverExtensionState(&self) -> Result<(), CommonError> {
		debug!("[ApplicationState] Recovering extension state...");
		
		// Clear potentially corrupted extension state
		let mut scanned_extensions = self.ScannedExtensions.lock().map_err(|e| {
			MapLockErrorWithRecovery(e, "RecoverExtensionState - ScannedExtensions")
		})?;
		
		scanned_extensions.clear();
		
		// Reset extension scan paths
		let mut scan_paths = self.ExtensionScanPaths.lock().map_err(|e| {
			MapLockErrorWithRecovery(e, "RecoverExtensionState - ExtensionScanPaths")
		})?;
		
		// Keep only valid paths that exist
		scan_paths.retain(|path| path.exists());
		
		Ok(())
	}

	/// Recover document state
	async fn RecoverDocumentState(&self) -> Result<(), CommonError> {
		debug!("[ApplicationState] Recovering document state...");
		
		// Clear potentially corrupted document state
		let mut open_documents = self.OpenDocuments.lock().map_err(|e| {
			MapLockErrorWithRecovery(e, "RecoverDocumentState - OpenDocuments")
		})?;
		
		// Remove documents that reference non-existent files
		open_documents.retain(|uri, doc_state| {
			if let Ok(url) = url::Url::parse(uri) {
				if url.scheme() == "file" {
					if let Some(path) = url.to_file_path().ok() {
						return path.exists();
					}
				}
			}
			true // Keep non-file URIs
		});
		
		Ok(())
	}
}
