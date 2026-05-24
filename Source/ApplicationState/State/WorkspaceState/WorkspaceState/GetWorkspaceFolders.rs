//! `WorkspaceState::GetWorkspaceFolders`

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

pub fn Fn(This:&Struct) -> Vec<WorkspaceFolderStateDTO> {
		This.WorkspaceFolders.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}
