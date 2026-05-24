//! `WorkspaceState::SetWindowState`

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

pub fn Fn(This:&Struct, state:WindowStateDTO) {
		if let Ok(mut guard) = This.WindowState.lock() {
			*guard = state;
			dev_log!("workspaces", "[WorkspaceState] Window state updated");
		}
	}
