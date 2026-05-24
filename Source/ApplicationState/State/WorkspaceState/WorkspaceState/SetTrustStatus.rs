//! `WorkspaceState::SetTrustStatus`

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

pub fn Fn(This:&Struct, trusted:bool) {
		This.IsTrusted.store(trusted, AtomicOrdering::Relaxed);

		dev_log!("workspaces", "[WorkspaceState] Trust status set to: {}", trusted);
	}
