//! `WorkspaceState::SetWorkspaceFolders`

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

pub fn Fn(This:&Struct, folders:Vec<WorkspaceFolderStateDTO>) {
		if let Ok(mut guard) = This.WorkspaceFolders.lock() {
			*guard = folders;
			dev_log!(
				"workspaces",
				"[WorkspaceState] Workspace folders updated ({} folders)",
				guard.len()
			);
		}
	}
