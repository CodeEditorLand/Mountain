//! `MarkerState::RemoveStatusBarItem`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{
		Arc,
		Mutex as StandardMutex,
		atomic::{AtomicU32, Ordering as AtomicOrdering},
	},
};
use CommonLibrary::{
	SourceControlManagement::DTO::{
		SourceControlManagementGroupDTO::SourceControlManagementGroupDTO,
		SourceControlManagementProviderDTO::SourceControlManagementProviderDTO,
		SourceControlManagementResourceDTO::SourceControlManagementResourceDTO,
	},
	StatusBar::DTO::StatusBarEntryDTO::StatusBarEntryDTO,
};
use crate::{ApplicationState::DTO::CustomDocumentStateDTO::CustomDocumentStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) {
		if let Ok(mut guard) = This.ActiveStatusBarItems.lock() {
			guard.remove(id);

			dev_log!("extensions", "[MarkerState] Status bar item removed: {}", id);
		}
	}
