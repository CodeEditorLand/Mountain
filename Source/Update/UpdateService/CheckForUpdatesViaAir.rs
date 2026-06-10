//! Delegate the update check to the Air gRPC service. Centralised update
//! management across the Land ecosystem; download path still TODO.

#[cfg(feature = "AirIntegration")]
use std::sync::Arc;

#[cfg(feature = "AirIntegration")]
<<<<<<< HEAD
use AirLibrary::Vine::Generated::air::{
	ApplyUpdateRequest,
	DownloadRequest,
	air_service_client::AirServiceClient,
	air_service_server::UpdateCheckRequest,
};
=======
use AirLibrary::Vine::Generated::air::{air_service_client::AirServiceClient, air_service_server::UpdateCheckRequest};
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
#[cfg(feature = "AirIntegration")]
use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, ShowMessage::ShowMessage},
};
#[cfg(feature = "AirIntegration")]
use serde_json::json;
#[cfg(feature = "AirIntegration")]
use tauri::{AppHandle, Emitter};

#[cfg(feature = "AirIntegration")]
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[cfg(feature = "AirIntegration")]
pub async fn Fn(
	ApplicationHandle:AppHandle,

	RunTime:Arc<Runtime>,

	NotifyNoUpdate:bool,

	AirClient:&Arc<AirServiceClient<tonic::transport::Channel>>,
) -> Result<(), CommonError> {
	dev_log!("update", "[UpdateService] Checking via Air...");

	let CurrentVersion = env!("CARGO_PKG_VERSION").to_string();

	let RequestID = uuid::Uuid::new_v4().to_string();

	let GrpcRequest = tonic::Request::new(UpdateCheckRequest {
		request_id:RequestID.clone(),
		current_version:CurrentVersion,
		channel:"stable".to_string(),
	});

	match AirClient.check_for_updates(GrpcRequest).await {
		Ok(Response) => {
			let Reply = Response.into_inner();

			if Reply.update_available {
				dev_log!("update", "[UpdateService] Air reports v{}", Reply.version);

				let Message = format!(
					"A new version of Mountain is available: v{}.\n\n{}",
					Reply.version, Reply.release_notes
				);

				let UserResponse = RunTime
					.Run(ShowMessage(
						MessageSeverity::Info,
						Message,
						json!({ "modal": true, "actions": ["Install", "Later"] }),
					))
					.await?;

				if UserResponse == Some("Install".to_string()) {
<<<<<<< HEAD
					let DownloadUrl = Reply.download_url.clone();

					let Version = Reply.version.clone();

					let DownloadDest = std::env::temp_dir()
						.join(format!("mountain-update-{}.pkg", Version))
						.to_string_lossy()
						.into_owned();

					let _ = ApplicationHandle.emit(
						"sky://update/downloading",
						json!({ "version": Version, "download_url": DownloadUrl }),
					);

					let DownloadReq = tonic::Request::new(DownloadRequest {
						request_id:RequestID.clone(),
						url:DownloadUrl,
						destination_path:DownloadDest.clone(),
						checksum:String::new(),
						headers:std::collections::HashMap::new(),
					});

					match AirClient.download_update(DownloadReq).await {
						Ok(DownloadResp) => {
							let DownloadReply = DownloadResp.into_inner();

							if !DownloadReply.success {
								RunTime
									.Run(ShowMessage(
										MessageSeverity::Error,
										format!("Update download failed: {}", DownloadReply.error),
										json!(null),
									))
									.await?;

								return Ok(());
							}

							let FilePath = DownloadReply.file_path;

							let _ = ApplicationHandle.emit("sky://update/downloaded", json!({ "version": Version }));

							let ApplyReq = tonic::Request::new(ApplyUpdateRequest {
								request_id:RequestID.clone(),
								version:Version.clone(),
								update_path:FilePath,
							});

							match AirClient.apply_update(ApplyReq).await {
								Ok(ApplyResp) => {
									let ApplyReply = ApplyResp.into_inner();

									if ApplyReply.success {
										RunTime
											.Run(ShowMessage(
												MessageSeverity::Info,
												format!(
													"Mountain v{} is ready. Please restart to apply the update.",
													Version
												),
												json!(null),
											))
											.await?;
									} else {
										RunTime
											.Run(ShowMessage(
												MessageSeverity::Error,
												format!("Update installation failed: {}", ApplyReply.error),
												json!(null),
											))
											.await?;
									}
								},

								Err(ApplyStatus) => {
									dev_log!(
										"update",
										"error: [UpdateService] apply_update RPC failed: {}",
										ApplyStatus
									);

									RunTime
										.Run(ShowMessage(
											MessageSeverity::Error,
											format!("Failed to apply update: {}", ApplyStatus),
											json!(null),
										))
										.await?;
								},
							}
						},

						Err(DownloadStatus) => {
							dev_log!(
								"update",
								"error: [UpdateService] download_update RPC failed: {}",
								DownloadStatus
							);

							RunTime
								.Run(ShowMessage(
									MessageSeverity::Error,
									format!("Failed to download update: {}", DownloadStatus),
									json!(null),
								))
								.await?;
						},
					}
=======
					// TODO: call Air's download_update endpoint, track progress, install.
					RunTime
						.Run(ShowMessage(
							MessageSeverity::Info,
							"Update download via Air is not yet implemented. Please update manually.".to_string(),
							json!(null),
						))
						.await?;
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
				}
			} else if NotifyNoUpdate {
				RunTime
					.Run(ShowMessage(
						MessageSeverity::Info,
						"You are running the latest version of Mountain.".to_string(),
						json!(null),
					))
					.await?;
			}

			Ok(())
		},

		Err(Status) => {
			dev_log!("update", "error: [UpdateService] Air update check failed: {}", Status);

			if NotifyNoUpdate {
				RunTime
					.Run(ShowMessage(
						MessageSeverity::Error,
						format!("Failed to check for updates via Air: {}", Status),
						json!(null),
					))
					.await?;
			}

			Err(CommonError::ExternalServiceError {
				ServiceName:"Air Update Service".to_string(),
				Description:Status.to_string(),
			})
		},
	}
}
