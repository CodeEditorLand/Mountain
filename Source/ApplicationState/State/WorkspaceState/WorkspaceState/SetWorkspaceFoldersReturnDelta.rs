//! `WorkspaceState::SetWorkspaceFoldersReturnDelta`

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

pub fn Fn(
		&self,

		folders:Vec<WorkspaceFolderStateDTO>,
	) -> (Vec<WorkspaceFolderStateDTO>, Vec<WorkspaceFolderStateDTO>) {
		match This.WorkspaceFolders.lock() {
			Ok(mut guard) => {
				let Old = guard.clone();

				let OldUris:std::collections::HashSet<String> = Old.iter().map(|F| F.URI.to_string()).collect();

				let NewUris:std::collections::HashSet<String> = folders.iter().map(|F| F.URI.to_string()).collect();

				let Added:Vec<WorkspaceFolderStateDTO> = folders
					.iter()
					.filter(|F| !OldUris.contains(&F.URI.to_string()))
					.cloned()
					.collect();

				let Removed:Vec<WorkspaceFolderStateDTO> =
					Old.iter().filter(|F| !NewUris.contains(&F.URI.to_string())).cloned().collect();

				*guard = folders;
				dev_log!(
					"workspaces",
					"[WorkspaceState] Workspace folders updated ({} folders, +{} -{})",
					guard.len(),
					Added.len(),
					Removed.len()
				);

				(Added, Removed)
			},

			Err(_) => (Vec::new(), Vec::new()),
		}
	}
