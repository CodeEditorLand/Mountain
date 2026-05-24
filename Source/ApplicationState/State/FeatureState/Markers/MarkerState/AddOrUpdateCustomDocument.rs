//! `MarkerState::AddOrUpdateCustomDocument`

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

pub fn Fn(This:&Struct, id:String, document:CustomDocumentStateDTO) {
		if let Ok(mut guard) = This.ActiveCustomDocuments.lock() {
			guard.insert(id, document);

			dev_log!("extensions", "[MarkerState] Custom document added/updated");
		}
	}
