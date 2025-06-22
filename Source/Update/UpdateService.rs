// File: Mountain/Source/Update/UpdateService.rs
// Role: Handles application update checking, user notification, and
// installation using the `tauri-plugin-updater`.

//! # Update Service
//!
//! Handles the application update checking and installation process using
//! `tauri-plugin-updater`.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, ShowMessage::ShowMessage},
};
use log::{error, info};
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

/// Checks for application updates, notifies the user if an update is found or
/// if an error occurs, and handles the download and installation process if
/// the user consents.
pub async fn CheckForUpdates(
	ApplicationHandle:AppHandle,

	RunTime:Arc<MountainRunTime>,

	NotifyNoUpdate:bool,
) -> Result<(), CommonError> {
	info!("[UpdateService] Checking for updates...");

	let updater = ApplicationHandle
		.updater_builder()
		.build()
		.map_err(|e| CommonError::ExternalServiceError { ServiceName:"Updater".into(), Description:e.to_string() })?;

	match updater.check().await {
		Ok(Some(update)) => {
			info!("Update available: v{} ({:?})", update.version, update.date);

			let update_notes = update.body.clone().unwrap_or_else(|| "No release notes provided.".to_string());

			let message = format!(
				"A new version of Mountain is available: v{}.\n\n{}",
				update.version, update_notes
			);

			// The `Options` parameter is a `Value`, not an `Option<Value>`.
			let user_response = RunTime
				.Run(ShowMessage(
					MessageSeverity::Info,
					message,
					json!({

						"modal": true,

						"actions": ["Install", "Later"]
					}),
				))
				.await?;

			if user_response == Some("Install".to_string()) {
				info!("[UpdateService] User chose to install. Downloading and installing...");

				let on_chunk = |chunk_size, total_size| {
					info!("[Update] Download progress: {} / {:?}", chunk_size, total_size);
				};

				let on_download_finish = || {
					info!("[Update] Download complete, starting installation.");
				};

				if let Err(e) = update.download_and_install(on_chunk, on_download_finish).await {
					error!("[UpdateService] Update failed: {}", e);

					RunTime
						.Run(ShowMessage(
							MessageSeverity::Error,
							format!("Failed to install update: {}", e),
							json!(null),
						))
						.await?;
				}
			} else {
				info!("[UpdateService] User chose not to install.");
			}
		},

		Ok(None) => {
			if NotifyNoUpdate {
				info!("[UpdateService] No updates available.");

				RunTime
					.Run(ShowMessage(
						MessageSeverity::Info,
						"You are running the latest version of Mountain.".to_string(),
						json!(null),
					))
					.await?;
			} else {
				info!("[UpdateService] No updates available (silent check).");
			}
		},

		Err(e) => {
			error!("[UpdateService] Failed to check for updates: {}", e);

			if NotifyNoUpdate {
				RunTime
					.Run(ShowMessage(
						MessageSeverity::Error,
						format!("Failed to check for updates: {}", e),
						json!(null),
					))
					.await?;
			}
		},
	}

	Ok(())
}
