//! `WorkspaceState::GetWindowState`

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

pub fn Fn(This:&Struct) -> WindowStateDTO {
		This.WorkspaceFolders
			.lock()
			.ok()
			.and_then(|_| This.WindowState.lock().ok().map(|guard| guard.clone()))
			.unwrap_or_default()
	}
