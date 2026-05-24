//! `WorkspaceState::GetActiveDocumentURI`

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

pub fn Fn(This:&Struct) -> Option<String> {
		This.ActiveDocumentURI.lock().ok().and_then(|guard| guard.clone())
	}
