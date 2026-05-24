//! `WorkspaceState::SetConfigurationPath`

use super::Struct;
use std::sync::{
	Arc,
	Mutex as StandardMutex,
	atomic::{AtomicBool, Ordering as AtomicOrdering},
};
use crate::{
	ApplicationState::DTO::{WindowStateDTO::WindowStateDTO, WorkspaceFolderStateDTO::WorkspaceFolderStateDTO},
	dev_log,
};

pub fn Fn(This:&Struct, path:Option<std::path::PathBuf>) {
		if let Ok(mut guard) = This.WorkspaceConfigurationPath.lock() {
			*guard = path.clone();
			dev_log!("workspaces", "[WorkspaceState] Configuration path updated to: {:?}", path);
		}
	}
