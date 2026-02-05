//! # UpdateService (Update)
//!
//! Handles the application update checking, user notification, and installation
//! process using `tauri-plugin-updater`, with optional delegation to the Air
//! service for centralized update management.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Update Checking
//! - Check for available updates using Tauri's updater system
//! - Support Air service delegation for centralized updates (when enabled)
//! - Compare versions and determine if update is available
//! - Handle network errors and service unavailability gracefully
//!
//! ### 2. User Interaction
//! - Notify user when updates are found (if `NotifyNoUpdate` is true)
//! - Prompt user for install confirmation with release notes
//! - Display error messages when update operations fail
//! - Support silent update checks (no UI when no update available)
//!
//! ### 3. Update Installation
//! - Download and install updates using Tauri's updater
//! - Show download progress during update acquisition
//! - Handle installation success and failure scenarios
//! - Support automatic restart after installation (via Tauri)
//!
//! ### 4. Air Integration (Optional)
//! - Delegate update checking to Air service when available
//! - Support `UpdateMode` configuration (AutoDetect, ForceAir, ForceTauri)
//! - Handle Air service unavailability with fallbacks
//! - Provide health checking for Air daemon connectivity
//!
//! ## ARCHITECTURAL ROLE
//!
//! UpdateService is the **update management layer** for Mountain:
//!
//! ```text
//! Binary (Startup) ──► UpdateService ──► Tauri Updater / Air Service ──► Install
//!                          │
//!                          └─► UI (ShowMessage) ──► User Consent
//! ```
//!
//! ### Position in Mountain
//! - `Update` module: Application update lifecycle
//! - Called from `Binary::Main` or manually by users (Help menu)
//! - Provides update capability via `Environment::UpdateProvider` trait
//!
//! ### Dependencies
//! - `tauri_plugin_updater`: Tauri's update mechanism
//! - `AirLibrary::Vine::Generated::Air` (optional): Air service client
//! - `CommonLibrary::UserInterface::ShowMessage`: User notifications
//! - `ApplicationRunTime`: Effect execution for UI operations
//!
//! ### Dependents
//! - `Binary::Main::Fn`: May trigger update check at startup
//! - Command handlers: Manual update check commands
//! - UI menu items: Help → Check for Updates
//!
//! ## UPDATE MODES
//!
//! The `UpdateMode` enum controls which update mechanism is used:
//!
//! - **AutoDetect** (default): Use Air if available, otherwise Tauri updater
//! - **ForceAir**: Use Air exclusively (errors if Air unavailable)
//! - **ForceTauri**: Use Tauri updater exclusively (ignore Air)
//!
//! ## ERROR HANDLING
//!
//! - Network errors: Logged and user notified (if `NotifyNoUpdate`)
//! - Update download failures: Error shown to user, retry possible
//! - Air service errors: Fallback to Tauri or fail based on mode
//! - Installation errors: Reported to user with diagnostic info
//!
//! ## PERFORMANCE
//!
//! - Update checks are async and non-blocking
//! - Download progress is reported in real-time via callbacks
//! - Air delegation adds network hop but enables centralized caching
//! - Tauri updater uses native platform APIs (NSIS, MSI, etc.)
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/platform/update/common/updateService.ts` - Update orchestration
//! - `vs/platform/update/electronbrowser/electronUpdater.ts` - Platform-specific updates
//! - `vs/workbench/services/extensions/common/extensionManagementService.ts` - Update notification
//!
//! ## TODO
//!
//! - [ ] Implement complete Air-based update download and installation
//! - [ ] Add digital signature verification for downloaded updates
//! - [ ] Implement delta updates to reduce download size
//! - [ ] Add update rollback capability
//! - [ ] Support custom update channels (stable, beta, nightly)
//! - [ ] Add update download resumption for interrupted downloads
//! - [ ] Implement staged rollout with percentage-based deployment
//! - [ ] Add telemetry for update success/failure rates
//! - [ ] Support background silent updates with user opt-out
//! - [ ] Add update scheduling (e.g., install at next restart)
//!
//! ## MODULE CONTENTS
//!
//! - [`CheckForUpdates`]: Primary update check using Tauri updater
//! - [`CheckForUpdatesWithAir`]: Update check with Air delegation support
//! - [`CheckForUpdatesViaAir`]: Air-based update implementation (feature-gated)
//! - `IsAirAvailable`: Helper to check Air service health
//! - [`UpdateMode`]: Update delegation mode enum
//!
//! ## EXAMPLE
//!
//! ```rust,no_run
//! use crate::Source::Update::UpdateService::{CheckForUpdates, CheckForUpdatesWithAir, UpdateMode};
//!
//! // Simple check using Tauri updater
//! CheckForUpdates(app_handle, runtime, true).await?;
//!
//! // Check with Air delegation (AutoDetect mode)
//! CheckForUpdatesWithAir(app_handle, runtime, true, Some(air_client), UpdateMode::AutoDetect).await?;
//! ```

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

use std::sync::Arc;

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, ShowMessage::ShowMessage},
};
use log::{error, info, warn};
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
// Import Air client types when Air is available in the workspace
#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::Air::AirServiceClient;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

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
/// - `UpdateMode::AutoDetect` (default): Use Air if available, otherwise use
///   Tauri updater
/// - `UpdateMode::ForceAir`: Use Air exclusively (returns error if Air
///   unavailable)
/// - `UpdateMode::ForceTauri`: Use Tauri updater exclusively
///
/// When Air is selected and available, delegates update checking to the Air
/// service. This enables centralized update management across all Land
/// applications.
///
/// # Arguments
/// * `ApplicationHandle` - The Tauri application handle
/// * `RunTime` - The Mountain runtime for UI interactions
/// * `NotifyNoUpdate` - Whether to notify the user when no updates are
///   available
/// * `AirClient` - Optional Air client for cloud-based update checking
/// * `Mode` - Update mode controlling delegation behavior
///
/// # Examples
/// ```rust,no_run
/// use crate::Source::Update::UpdateService::{CheckForUpdatesWithAir, UpdateMode};
///
/// // Auto-detect: Use Air if available
/// CheckForUpdatesWithAir(app_handle, runtime, true, Some(air_client), UpdateMode::AutoDetect)
/// 	.await?;
///
/// // Force Air usage
/// CheckForUpdatesWithAir(app_handle, runtime, true, Some(air_client), UpdateMode::ForceAir)
/// 	.await?;
///
/// // Force local Tauri updater
/// CheckForUpdatesWithAir(app_handle, runtime, true, None, UpdateMode::ForceTauri).await?;
/// ```
#[cfg(not(feature = "AirIntegration"))]
pub async fn CheckForUpdatesWithAir(
	ApplicationHandle:AppHandle,
	RunTime:Arc<MountainRunTime>,
	NotifyNoUpdate:bool,
	_AirClient:Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
	Mode:UpdateMode,
) -> Result<(), CommonError> {
	match Mode {
		UpdateMode::ForceAir => {
			error!("[UpdateService] ForceAir mode specified but Air integration is disabled");
			return Err(CommonError::Configuration {
				Message:"Air integration is not enabled. Build with `--features AirIntegration` to use ForceAir mode."
					.to_string(),
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
/// - `UpdateMode::AutoDetect` (default): Use Air if available, otherwise use
///   Tauri updater
/// - `UpdateMode::ForceAir`: Use Air exclusively (returns error if Air
///   unavailable)
/// - `UpdateMode::ForceTauri`: Use Tauri updater exclusively
///
/// When Air is selected and available, delegates update checking to the Air
/// service. This enables centralized update management across all Land
/// applications.
///
/// # Arguments
/// * `ApplicationHandle` - The Tauri application handle
/// * `RunTime` - The Mountain runtime for UI interactions
/// * `NotifyNoUpdate` - Whether to notify the user when no updates are
///   available
/// * `AirClient` - Optional Air client for cloud-based update checking
/// * `Mode` - Update mode controlling delegation behavior
///
/// # Examples
/// ```rust,no_run
/// use crate::Source::Update::UpdateService::{CheckForUpdatesWithAir, UpdateMode};
///
/// // Auto-detect: Use Air if available
/// CheckForUpdatesWithAir(app_handle, runtime, true, Some(air_client), UpdateMode::AutoDetect)
/// 	.await?;
///
/// // Force Air usage
/// CheckForUpdatesWithAir(app_handle, runtime, true, Some(air_client), UpdateMode::ForceAir)
/// 	.await?;
///
/// // Force local Tauri updater
/// CheckForUpdatesWithAir(app_handle, runtime, true, None, UpdateMode::ForceTauri).await?;
/// ```
#[cfg(feature = "AirIntegration")]
pub async fn CheckForUpdatesWithAir(
	ApplicationHandle:AppHandle,
	RunTime:Arc<MountainRunTime>,
	NotifyNoUpdate:bool,
	AirClient:Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
	Mode:UpdateMode,
) -> Result<(), CommonError> {
	match Mode {
		UpdateMode::ForceAir => {
			info!("[UpdateService] ForceAir mode specified - requiring Air service");

			let AirClientRef = AirClient.as_ref().ok_or_else(|| {
				CommonError::Configuration { Message:"ForceAir mode requires a valid AirClient".to_string() }
			})?;

			return CheckForUpdatesViaAir(ApplicationHandle, RunTime, NotifyNoUpdate, AirClientRef).await;
		},

		UpdateMode::ForceTauri => {
			info!("[UpdateService] ForceTauri mode specified - using Tauri updater");
			return CheckForUpdates(ApplicationHandle, RunTime, NotifyNoUpdate).await;
		},

		UpdateMode::AutoDetect => {
			if let Some(AirClientRef) = &AirClient {
				if IsAirAvailable(AirClientRef).await {
					info!("[UpdateService] Air service available - delegating update check to Air");
					return CheckForUpdatesViaAir(ApplicationHandle, RunTime, NotifyNoUpdate, AirClientRef).await;
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
	ApplicationHandle:AppHandle,
	RunTime:Arc<MountainRunTime>,
	NotifyNoUpdate:bool,
	AirClient:&Arc<AirServiceClient<tonic::transport::Channel>>,
) -> Result<(), CommonError> {
	info!("[UpdateService] Checking for updates via Air service...");

	use tonic::Request;

	let CurrentVersion = env!("CARGO_PKG_VERSION").to_string();
	let RequestID = uuid::Uuid::new_v4().to_string();

	let Request = tonic::Request::new(air_service_server::UpdateCheckRequest {
		request_id:RequestID,
		current_version:CurrentVersion,
		channel:"stable".to_string(),
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

					// Download and install updates via the Air service after user confirmation.
					// Call Air's download_update endpoint to fetch the update package, track
					// download progress, and then execute the platform-specific installation.
					// The Air service handles update packaging, signature verification, and
					// provides progress feedback. Currently showing a placeholder message.
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
					ServiceName:"Air Update Service".to_string(),
					Description:Status.to_string(),
				});
			};

			RunTime
				.Run(ShowMessage(MessageSeverity::Error, error_message, json!(null)))
				.await?;

			Err(CommonError::ExternalServiceError {
				ServiceName:"Air Update Service".to_string(),
				Description:Status.to_string(),
			})
		},
	}
}

/// Helper to check if Air service is available and healthy.
#[cfg(feature = "AirIntegration")]
async fn IsAirAvailable(AirClient:&AirServiceClient<tonic::transport::Channel>) -> bool {
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
