use Common::{error::CommonError, status_bar::dto::StatusBarEntryDto};
use log::info;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// @module StatusBarLogic
/// @description Contains the core logic for managing status bar items. It
/// handles RPC calls from Cocoon and emits events to the Sky frontend for
/// rendering.
use crate::AppState::AppState::AppState;

/// Logic to create a new status bar item or update an existing one. This is
/// called by the `StatusBarProvider` in the environment.
///
/// @param Entry - The DTO containing the complete state of the status bar item
/// to be rendered.
pub async fn SetEntryLogic<R:Runtime>(AppHandle:&AppHandle<R>, Entry:StatusBarEntryDto) -> Result<(), CommonError> {
	info!("[StatusBarLogic] Setting entry with ID: {}", Entry.EntryId);
	let AppStateInstance = AppHandle.state::<AppState>();

	// Store the latest state of the item in our central state map.
	AppStateInstance
		.ActiveStatusBarItems
		.lock()
		.unwrap()
		.insert(Entry.EntryId.clone(), Entry.clone());

	// Notify the Sky frontend to render or update the item in the UI.
	AppHandle
		.emit("sky://statusbar/set-entry", Entry)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

/// Logic to remove a status bar item from the UI. This is called by the
/// `StatusBarProvider` when an extension disposes of an item.
///
/// @param EntryId - The unique identifier of the entry to remove.
pub async fn DisposeEntryLogic<R:Runtime>(AppHandle:&AppHandle<R>, EntryId:String) -> Result<(), CommonError> {
	info!("[StatusBarLogic] Disposing entry with ID: {}", EntryId);
	let AppStateInstance = AppHandle.state::<AppState>();

	// Remove the item from our central state.
	AppStateInstance.ActiveStatusBarItems.lock().unwrap().remove(&EntryId);

	// Notify the Sky frontend to remove the item from the UI.
	AppHandle
		.emit("sky://statusbar/dispose-entry", json!({ "EntryId": EntryId }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
