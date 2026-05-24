//! `WorkspaceState::GetConfigurationPath`

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

pub fn Fn(This:&Struct) -> Option<std::path::PathBuf> {
		This.WorkspaceConfigurationPath.lock().ok().and_then(|guard| guard.clone())
	}
