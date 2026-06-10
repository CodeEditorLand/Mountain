//! Update dispatcher.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Update::{
	ApplyUpdate::Fn as UpdateApplyUpdate,
	CheckForUpdates::Fn as UpdateCheckForUpdates,
	DownloadUpdate::Fn as UpdateDownloadUpdate,
	GetInitialState::Fn as UpdateGetInitialState,
	IsLatestVersion::Fn as UpdateIsLatestVersion,
	QuitAndInstall::Fn as UpdateQuitAndInstall,
};

/// Dispatches update commands.
pub async fn dispatch_update(command:&str) -> Result<Value, String> {
	match command {
		"update:_getInitialState" => UpdateGetInitialState().await,

		"update:isLatestVersion" => UpdateIsLatestVersion().await,

		"update:checkForUpdates" => UpdateCheckForUpdates().await,

		"update:downloadUpdate" => UpdateDownloadUpdate().await,

		"update:applyUpdate" => UpdateApplyUpdate().await,

		"update:quitAndInstall" => UpdateQuitAndInstall().await,

		_ => Err(format!("Unknown update command: {}", command)),
	}
}
