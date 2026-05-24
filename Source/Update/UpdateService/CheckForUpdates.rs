//! Check for updates using Tauri's bundled updater. Notifies the user, asks
//! for Install consent, and runs `download_and_install` on accept.
//!
//! ## Status
//!
//! Zero call sites as of 2026-05-02. Wire from `Binary::Main` (Help
//! Check for Updates) or remove entirely if Air is the canonical path.

use std::sync::Arc;

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, ShowMessage::ShowMessage},
};
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

pub async fn Fn(ApplicationHandle:AppHandle, RunTime:Arc<Runtime>, NotifyNoUpdate:bool) -> Result<(), CommonError> {
	dev_log!("update", "[UpdateService] Checking for updates...");

	let Updater = ApplicationHandle.updater_builder().build().map_err(|Error| {
		CommonError::ExternalServiceError { ServiceName:"Updater".into(), Description:Error.to_string() }
	})?;

	match Updater.check().await {
		Ok(Some(Update)) => {
			dev_log!("update", "Update available: v{} ({:?})", Update.version, Update.date);

			let Notes = Update.body.clone().unwrap_or_else(|| "No release notes provided.".to_string());

			let Message = format!("A new version of Mountain is available: v{}.\n\n{}", Update.version, Notes);

			let Response = RunTime
				.Run(ShowMessage(
					MessageSeverity::Info,
					Message,
					json!({ "modal": true, "actions": ["Install", "Later"] }),
				))
				.await?;

			if Response == Some("Install".to_string()) {
				dev_log!("update", "[UpdateService] User chose to Install. Downloading...");

				let OnChunk = |Bytes, Total| {
					dev_log!("update", "[Update] progress {} / {:?}", Bytes, Total);
				};

				let OnFinish = || {
					dev_log!("update", "[Update] download complete; installing");
				};

				if let Err(Error) = Update.download_and_install(OnChunk, OnFinish).await {
					dev_log!("update", "error: [UpdateService] Install failed: {}", Error);

					RunTime
						.Run(ShowMessage(
							MessageSeverity::Error,
							format!("Failed to Install update: {}", Error),
							json!(null),
						))
						.await?;
				}
			} else {
				dev_log!("update", "[UpdateService] User declined Install.");
			}
		},

		Ok(None) => {
			if NotifyNoUpdate {
				RunTime
					.Run(ShowMessage(
						MessageSeverity::Info,
						"You are running the latest version of Mountain.".to_string(),
						json!(null),
					))
					.await?;
			}
		},

		Err(Error) => {
			dev_log!("update", "error: [UpdateService] check failed: {}", Error);

			if NotifyNoUpdate {
				RunTime
					.Run(ShowMessage(
						MessageSeverity::Error,
						format!("Failed to check for updates: {}", Error),
						json!(null),
					))
					.await?;
			}
		},
	}

	Ok(())
}
