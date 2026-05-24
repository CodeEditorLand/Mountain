//! `WorkspaceState::SetActiveDocumentURI`

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

pub fn Fn(This:&Struct, uri:Option<String>) {
		if let Ok(mut guard) = This.ActiveDocumentURI.lock() {
			*guard = uri.clone();
			dev_log!("workspaces", "[WorkspaceState] Active document URI updated to: {:?}", uri);
		}
	}
