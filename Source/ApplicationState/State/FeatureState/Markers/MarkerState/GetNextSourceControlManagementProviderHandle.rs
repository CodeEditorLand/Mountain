//! `MarkerState::GetNextSourceControlManagementProviderHandle`

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

pub fn Fn(This:&Struct) -> u32 {
		This.NextSourceControlManagementProviderHandle
			.fetch_add(1, AtomicOrdering::Relaxed)
	}
