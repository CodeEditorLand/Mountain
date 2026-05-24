pub mod GetTrustStatus;
pub mod SetTrustStatus;
pub mod GetConfigurationPath;
pub mod SetConfigurationPath;
pub mod GetActiveDocumentURI;
pub mod SetActiveDocumentURI;
pub mod GetWorkspaceFolders;
pub mod SetWorkspaceFolders;
pub mod SetWorkspaceFoldersReturnDelta;
pub mod GetWindowState;
pub mod SetWindowState;

use std::sync::{
	Arc,
	Mutex as StandardMutex,
	atomic::{AtomicBool, Ordering as AtomicOrdering},
};
use crate::{
	ApplicationState::DTO::{WindowStateDTO::WindowStateDTO, WorkspaceFolderStateDTO::WorkspaceFolderStateDTO},
	dev_log,
};

/// Workspace state containing all workspace-related fields.
#[derive(Clone)]
pub struct Struct {
	/// Currently open workspace folders.
	pub WorkspaceFolders:Arc<StandardMutex<Vec<WorkspaceFolderStateDTO>>>,

	/// Path to the workspace configuration file (if any).
	pub WorkspaceConfigurationPath:Arc<StandardMutex<Option<std::path::PathBuf>>>,

	/// Workspace trust status (security).
	pub IsTrusted:Arc<AtomicBool>,

	/// Main window presentation state.
	pub WindowState:Arc<StandardMutex<WindowStateDTO>>,

	/// Currently active document URI.
	pub ActiveDocumentURI:Arc<StandardMutex<Option<String>>>,
}
