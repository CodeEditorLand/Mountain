pub mod GetNextSourceControlManagementProviderHandle;
pub mod GetCustomDocuments;
pub mod AddOrUpdateCustomDocument;
pub mod RemoveCustomDocument;
pub mod GetStatusBarItems;
pub mod AddOrUpdateStatusBarItem;
pub mod RemoveStatusBarItem;

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

/// Marker-related state containing custom documents, status bar, and SCM state.
#[derive(Clone)]
pub struct Struct {
	/// Active custom documents organized by ID.
	pub ActiveCustomDocuments:Arc<StandardMutex<HashMap<String, CustomDocumentStateDTO>>>,

	/// Active status bar items organized by ID.
	pub ActiveStatusBarItems:Arc<StandardMutex<HashMap<String, StatusBarEntryDTO>>>,

	/// SCM providers organized by handle.
	pub SourceControlManagementProviders:Arc<StandardMutex<HashMap<u32, SourceControlManagementProviderDTO>>>,

	/// SCM groups organized by provider handle and group ID.
	pub SourceControlManagementGroups:
		Arc<StandardMutex<HashMap<u32, HashMap<String, SourceControlManagementGroupDTO>>>>,

	/// SCM resources organized by provider handle and group ID.
	pub SourceControlManagementResources:
		Arc<StandardMutex<HashMap<u32, HashMap<String, Vec<SourceControlManagementResourceDTO>>>>>,

	/// Counter for generating unique SCM provider handles.
	pub NextSourceControlManagementProviderHandle:Arc<AtomicU32>,
}
