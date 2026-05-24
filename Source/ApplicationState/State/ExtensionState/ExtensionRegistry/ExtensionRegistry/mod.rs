pub mod GetNextProviderHandle;
pub mod GetCommands;
pub mod RegisterCommand;
pub mod UnregisterCommand;
pub mod GetExtensionScanPaths;
pub mod SetExtensionScanPaths;
pub mod AddExtensionScanPath;
pub mod GetEnabledProposedAPIs;
pub mod SetEnabledProposedAPIs;
pub mod EnableProposedAPI;

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{
		Arc,
		Mutex as StandardMutex,
		atomic::{AtomicU32, Ordering as AtomicOrdering},
	},
};
use tauri::Wry;
use crate::{Environment::CommandProvider::CommandHandler, dev_log};

/// Extension registry containing command registry and provider handle state.
#[derive(Clone)]
pub struct Struct {
	/// Registered CLI commands.
	pub CommandRegistry:Arc<StandardMutex<HashMap<String, CommandHandler<Wry>>>>,

	/// Counter for generating unique provider handles.
	pub NextProviderHandle:Arc<AtomicU32>,

	/// Paths to scan for extensions.
	pub ExtensionScanPaths:Arc<StandardMutex<Vec<PathBuf>>>,

	/// Enabled proposed APIs for extensions.
	pub EnabledProposedAPIs:Arc<StandardMutex<HashMap<String, Vec<String>>>>,
}
