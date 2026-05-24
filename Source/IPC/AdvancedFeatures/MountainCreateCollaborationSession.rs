//! `MountainCreateCollaborationSession` Tauri command -
//! create a fresh `CollaborationSession::Struct` with the
//! requested permissions.

use tauri::Manager;

use crate::{
	IPC::AdvancedFeatures::{
		CollaborationPermissions::Struct as CollaborationPermissions,
		Features::Struct as Features,
	},
	dev_log,
};

#[tauri::command]
pub async fn Fn(
	app_handle:tauri::AppHandle,

	SessionId:String,

	permissions:CollaborationPermissions,
) -> Result<(), String> {
	dev_log!("lifecycle", "Tauri command: create_collaboration_session");

	if let Some(features) = app_handle.try_state::<Features>() {
		features.CreateCollaborationSession(SessionId, permissions).await
	} else {
		Err("AdvancedFeatures not found in application state".to_string())
	}
}
