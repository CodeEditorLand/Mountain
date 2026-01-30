// File: Mountain/Source/Update/UpdateService.rs
// Role: Handles application update checking, user notification, and
// installation using the `tauri-plugin-updater`.

//! # Update Service
//!
//! Handles the application update checking and installation process using
//! `tauri-plugin-updater`.
//!
//! ## Air Integration Strategy
//!
//! This service supports delegation to the Air service for update management:
//! - When ForceAir is true, uses Air exclusively (panics if unavailable)
//! - When AirClient is provided and available, delegates to Air for updates
//! - Falls back to Tauri updater when Air is unavailable
//!
//! TODO: Full Air Migration Plan
//! ============================
//! - [ ] Implement complete Air-based update management
//! - [ ] Add update download via Air service
//! - [ ] Implement update verification through Air
//! - [ ] Add support for scheduled updates via Air
//! - [ ] Implement rollback capability with Air
//! - [ ] Add metrics for Air vs Local update usage tracking
//! - [ ] Phase out Tauri updater after successful Air deployment

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, ShowMessage::ShowMessage},
};
use log::{error, info, warn};
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

// Import Air client types when Air is available in the workspace
#[cfg(feature = "AirIntegration")]
use Air::Vine::Generated::air::AirServiceClient;

/// Update delegation mode for controlling which update mechanism to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
	/// Auto-detect: Use Air if available, otherwise use Tauri updater
	AutoDetect,

	/// Force Air exclusively (panics if Air unavailable)
	ForceAir,

	/// Force Tauri updater exclusively
	ForceTauri,
}

impl Default for UpdateMode {
	fn default() -> Self { Self::AutoDetect }
}

/// Checks for application updates, notifies the user if an update is found or
/// if an error occurs, and handles the download and installation process if
/// the user consents.
pub async fn CheckForUpdates(
	ApplicationHandle:AppHandle,

	RunTime:Arc<MountainRunTime>,

	NotifyNoUpdate:bool,
) -> Result<(), CommonError> {
	info!("[UpdateService] Checking for updates...");

	let updater = ApplicationHandle.updater_builder().build().map_err(|Error| {
		CommonError::ExternalServiceError { ServiceName:"Updater".into(), Description:Error.to_string() }
	})?;

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

				if let Err(Error) = update.download_and_install(on_chunk, on_download_finish).await {
					error!("[UpdateService] Update failed: {}", Error);

					RunTime
						.Run(ShowMessage(
							MessageSeverity::Error,
							format!("Failed to install update: {}", Error),
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

		Err(Error) => {
			error!("[UpdateService] Failed to check for updates: {}", Error);

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

/// Checks for application updates with Air delegation support.
///
/// This function supports multiple update strategies via the `Mode` parameter:
/// - `UpdateMode::AutoDetect` (default): Use Air if available, otherwise use Tauri updater
/// - `UpdateMode::ForceAir`: Use Air exclusively (returns error if Air unavailable)
/// - `UpdateMode::ForceTauri`: Use Tauri updater exclusively
///
/// When Air is selected and available, delegates update checking to the Air service.
/// This enables centralized update management across all Land applications.
///
/// # Arguments
/// * `ApplicationHandle` - The Tauri application handle
/// * `RunTime` - The Mountain runtime for UI interactions
/// * `NotifyNoUpdate` - Whether to notify the user when no updates are available
/// * `AirClient` - Optional Air client for cloud-based update checking
/// * `Mode` - Update mode controlling delegation behavior
///
/// # Examples
/// ```rust,no_run
/// use crate::Source::Update::UpdateService::{CheckForUpdatesWithAir, UpdateMode};
///
/// // Auto-detect: Use Air if available
/// CheckForUpdatesWithAir(
///     app_handle,
///     runtime,
///     true,
///     Some(air_client),
///     UpdateMode::AutoDetect,
/// ).await?;
///
/// // Force Air usage
/// CheckForUpdatesWithAir(
///     app_handle,
///     runtime,
///     true,
///     Some(air_client),
///     UpdateMode::ForceAir,
/// ).await?;
///
/// // Force local Tauri updater
/// CheckForUpdatesWithAir(
///     app_handle,
///     runtime,
///     true,
///     None,
///     UpdateMode::ForceTauri,
/// ).await?;
/// ```
#[cfg(not(feature = "AirIntegration"))]
pub async fn CheckForUpdatesWithAir(
	ApplicationHandle: AppHandle,
	RunTime: Arc<MountainRunTime>,
	NotifyNoUpdate: bool,
	_AirClient: Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
	Mode: UpdateMode,
) -> Result<(), CommonError> {
	match Mode {
		UpdateMode::ForceAir => {
			error!(
				"[UpdateService] ForceAir mode specified but Air integration is disabled"
			);
			return Err(CommonError::Configuration {
				Message: "Air integration is not enabled. Build with `--features AirIntegration` to use ForceAir mode.".to_string(),
			});
		},
		UpdateMode::AutoDetect | UpdateMode::ForceTauri => {
			info!("[UpdateService] Using Tauri updater (Air integration disabled)");
		},
	}

	// Always use Tauri updater when Air integration is disabled
	CheckForUpdates(ApplicationHandle, RunTime, NotifyNoUpdate).await
}

/// Checks for application updates with Air delegation support.
///
/// This function supports multiple update strategies via the `Mode` parameter:
/// - `UpdateMode::AutoDetect` (default): Use Air if available, otherwise use Tauri updater
/// - `UpdateMode::ForceAir`: Use Air exclusively (returns error if Air unavailable)
/// - `UpdateMode::ForceTauri`: Use Tauri updater exclusively
///
/// When Air is selected and available, delegates update checking to the Air service.
/// This enables centralized update management across all Land applications.
///
/// # Arguments
/// * `ApplicationHandle` - The Tauri application handle
/// * `RunTime` - The Mountain runtime for UI interactions
/// * `NotifyNoUpdate` - Whether to notify the user when no updates are available
/// * `AirClient` - Optional Air client for cloud-based update checking
/// * `Mode` - Update mode controlling delegation behavior
///
/// # Examples
/// ```rust,no_run
/// use crate::Source::Update::UpdateService::{CheckForUpdatesWithAir, UpdateMode};
///
/// // Auto-detect: Use Air if available
/// CheckForUpdatesWithAir(
///     app_handle,
///     runtime,
///     true,
///     Some(air_client),
///     UpdateMode::AutoDetect,
/// ).await?;
///
/// // Force Air usage
/// CheckForUpdatesWithAir(
///     app_handle,
///     runtime,
///     true,
///     Some(air_client),
///     UpdateMode::ForceAir,
/// ).await?;
///
/// // Force local Tauri updater
/// CheckForUpdatesWithAir(
///     app_handle,
///     runtime,
///     true,
///     None,
///     UpdateMode::ForceTauri,
/// ).await?;
/// ```
#[cfg(feature = "AirIntegration")]
pub async fn CheckForUpdatesWithAir(
	ApplicationHandle: AppHandle,
	RunTime: Arc<MountainRunTime>,
	NotifyNoUpdate: bool,
	AirClient: Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
	Mode: UpdateMode,
) -> Result<(), CommonError> {
	match Mode {
		UpdateMode::ForceAir => {
			info!("[UpdateService] ForceAir mode specified - requiring Air service");

			let AirClientRef = AirClient
				.as_ref()
				.ok_or_else(|| CommonError::Configuration {
					Message: "ForceAir mode requires a valid AirClient".to_string(),
				})?;

			return CheckForUpdatesViaAir(
				ApplicationHandle,
				RunTime,
				NotifyNoUpdate,
				AirClientRef,
			)
			.await;
		},

		UpdateMode::ForceTauri => {
			info!("[UpdateService] ForceTauri mode specified - using Tauri updater");
			return CheckForUpdates(ApplicationHandle, RunTime, NotifyNoUpdate).await;
		},

		UpdateMode::AutoDetect => {
			if let Some(AirClientRef) = &AirClient {
				if IsAirAvailable(AirClientRef).await {
					info!("[UpdateService] Air service available - delegating update check to Air");
					return CheckForUpdatesViaAir(
						ApplicationHandle,
						RunTime,
						NotifyNoUpdate,
						AirClientRef,
					)
					.await;
				} else {
					warn!("[UpdateService] Air client provided but unhealthy - falling back to Tauri updater");
				}
			} else {
				info!("[UpdateService] No Air client provided - using Tauri updater");
			}

			CheckForUpdates(ApplicationHandle, RunTime, NotifyNoUpdate).await
		},
	}
}

/// Checks for updates via the Air service.
///
/// This function delegates all update checking to the Air service, enabling
/// centralized update management across the Land ecosystem.
#[cfg(feature = "AirIntegration")]
async fn CheckForUpdatesViaAir(
	ApplicationHandle: AppHandle,
	RunTime: Arc<MountainRunTime>,
	NotifyNoUpdate: bool,
	AirClient: &Arc<AirServiceClient<tonic::transport::Channel>>,
) -> Result<(), CommonError> {
	info!("[UpdateService] Checking for updates via Air service...");

	use tonic::Request;

	let CurrentVersion = env!("CARGO_PKG_VERSION").to_string();
	let RequestID = uuid::Uuid::new_v4().to_string();

	let Request = tonic::Request::new(air_service_server::UpdateCheckRequest {
		request_id: RequestID,
		current_version: CurrentVersion,
		channel: "stable".to_string(),
	});

	match AirClient.check_for_updates(Request).await {
		Ok(Response) => {
			let UpdateCheckResponse = Response.into_inner();

			if UpdateCheckResponse.update_available {
				info!("[UpdateService] Air reports update available: v{}", UpdateCheckResponse.version);

				let message = format!(
					"A new version of Mountain is available: v{}.\n\n{}",
					UpdateCheckResponse.version, UpdateCheckResponse.release_notes
				);

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
					info!("[UpdateService] User chose to install via Air");

					// TODO: Implement download via Air service
					// This would involve calling Air's download_update endpoint
					RunTime
						.Run(ShowMessage(
							MessageSeverity::Info,
							"Update download via Air is not yet implemented. Please update manually.".to_string(),
							json!(null),
						))
						.await?;
				} else {
					info!("[UpdateService] User chose not to install");
				}
			} else {
				if NotifyNoUpdate {
					info!("[UpdateService] Air reports no updates available");

					RunTime
						.Run(ShowMessage(
							MessageSeverity::Info,
							"You are running the latest version of Mountain.".to_string(),
							json!(null),
						))
						.await?;
				} else {
					info!("[UpdateService] Air reports no updates available (silent check)");
				}
			}

			Ok(())
		},

		Err(Status) => {
			error!("[UpdateService] Air update check failed: {}", Status);

			let error_message = if NotifyNoUpdate {
				format!("Failed to check for updates via Air: {}", Status)
			} else {
				// Silent error - just log it
				return Err(CommonError::ExternalServiceError {
					ServiceName: "Air Update Service".to_string(),
					Description: Status.to_string(),
				});
			};

			RunTime
				.Run(ShowMessage(MessageSeverity::Error, error_message, json!(null)))
				.await?;

			Err(CommonError::ExternalServiceError {
				ServiceName: "Air Update Service".to_string(),
				Description: Status.to_string(),
			})
		},
	}
}

/// Helper to check if Air service is available and healthy.
#[cfg(feature = "AirIntegration")]
async fn IsAirAvailable(AirClient: &AirServiceClient<tonic::transport::Channel>) -> bool {
	use tonic::Request;

	match AirClient
		.health_check(Request::new(air_service_server::HealthCheckRequest {}))
		.await
	{
		Ok(Response) => {
			let is_healthy = Response.into_inner().healthy;

			if !is_healthy {
				warn!("[UpdateService] Air health check returned unhealthy");
			}

			is_healthy
		},

		Err(Error) => {
			warn!("[UpdateService] Air health check failed: {}", Error);
			false
		},
	}
}
