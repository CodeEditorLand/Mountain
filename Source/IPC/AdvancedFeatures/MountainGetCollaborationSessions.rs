//! `MountainGetCollaborationSessions` Tauri command -
//! returns every active `CollaborationSession::Struct`.

use tauri::Manager;

use crate::{
	IPC::AdvancedFeatures::{CollaborationSession::Struct as CollaborationSession, Features::Struct as Features},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:tauri::AppHandle) -> Result<Vec<CollaborationSession>, String> {
	dev_log!("lifecycle", "Tauri command: get_collaboration_sessions");

	if let Some(features) = app_handle.try_state::<Features>() {
		Ok(features.GetCollaborationSessions().await)
	} else {
		Err("AdvancedFeatures not found in application state".to_string())
	}
}
