// @module StatusBarLogic
// @description Contains the core logic for managing status bar items. It
// handles RPC calls from Cocoon and emits events to the Sky frontend for
// rendering.

use Common::{error::CommonError, status_bar::DTO::StatusBarEntryDto};
use log::info;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::ApplicationState::ApplicationState::ApplicationState;

/// Logic to create a new status bar item or update an existing one. This is
/// called by the `StatusBarProvider` in the Environment.
///
/// @param entry - The DTO containing the complete state of the status bar item
/// to be rendered.
pub async fn SetEntryLogic<R:Runtime>(app_handle:&AppHandle<R>, entry:StatusBarEntryDto) -> Result<(), CommonError> {
	info!("[StatusBarLogic] Setting entry with ID: {}", entry.EntryId);
	let app_state = app_handle.state::<ApplicationState>();

	// Store the latest state of the item in our central state map.
	app_state
		.ActiveStatusBarItems
		.lock()
		.unwrap()
		.insert(entry.EntryId.clone(), entry.clone());

	// Notify the Sky frontend to render or update the item in the UI.
	app_handle
		.emit("sky://statusbar/set-entry", entry)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

/// Logic to remove a status bar item from the UI. This is called by the
/// `StatusBarProvider` when an extension disposes of an item.
///
/// @param entry_id - The unique identifier of the entry to remove.
pub async fn DisposeEntryLogic<R:Runtime>(app_handle:&AppHandle<R>, entry_id:String) -> Result<(), CommonError> {
	info!("[StatusBarLogic] Disposing entry with ID: {}", entry_id);
	let app_state = app_handle.state::<ApplicationState>();

	// Remove the item from our central state.
	app_state.ActiveStatusBarItems.lock().unwrap().remove(&entry_id);

	// Notify the Sky frontend to remove the item from the UI.
	app_handle
		.emit("sky://statusbar/dispose-entry", json!({ "EntryId": entry_id }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
