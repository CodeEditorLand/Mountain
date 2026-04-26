//! # StatusBarProvider - Entry Management
//!
//! Implementation of status bar entry creation and disposal for
//! [`MountainEnvironment`]

use CommonLibrary::{
	Error::CommonError::CommonError,
	IPC::SkyEvent::SkyEvent,
	StatusBar::DTO::StatusBarEntryDTO::StatusBarEntryDTO,
};
use serde_json::json;
use tauri::Emitter;

use super::super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::dev_log;

/// Entry management operations implementation for MountainEnvironment
pub(super) async fn set_status_bar_entry_impl(
	env:&MountainEnvironment,
	entry:StatusBarEntryDTO,
) -> Result<(), CommonError> {
	dev_log!("lifecycle", "[StatusBarProvider] Setting entry: {}", entry.EntryIdentifier);

	let mut items_guard = env
		.ApplicationState
		.Feature
		.Markers
		.ActiveStatusBarItems
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	items_guard.insert(entry.EntryIdentifier.clone(), entry.clone());

	drop(items_guard);

	env.ApplicationHandle
		.emit(SkyEvent::StatusBarSetEntry.AsStr(), entry)
		.map_err(|error| CommonError::UserInterfaceInteraction { Reason:error.to_string() })
}

/// Removes a status bar item from the UI.
pub(super) async fn dispose_status_bar_entry_impl(
	env:&MountainEnvironment,
	entry_identifier:String,
) -> Result<(), CommonError> {
	dev_log!("lifecycle", "[StatusBarProvider] Disposing entry: {}", entry_identifier);

	env.ApplicationState
		.Feature
		.Markers
		.ActiveStatusBarItems
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.remove(&entry_identifier);

	env.ApplicationHandle
		.emit(
			SkyEvent::StatusBarDisposeEntry.AsStr(),
			json!({ "EntryIdentifier": entry_identifier }),
		)
		.map_err(|error| CommonError::UserInterfaceInteraction { Reason:error.to_string() })
}
