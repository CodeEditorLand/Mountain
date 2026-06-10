//! Delegate the update check to the Air gRPC service. Centralised update
//! management across the Land ecosystem; download path still TODO.

#[cfg(feature = "AirIntegration")]
use std::sync::Arc;

#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::air::{
	ApplyUpdateRequest,
	DownloadRequest,
	UpdateCheckRequest,
	air_service_client::AirServiceClient,
};
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
					let _ = ApplicationHandle.emit("sky://update/downloading", json!({ "version": Reply.version }));

					let mut Client = (**AirClient).clone();

					let DownloadReq = tonic::Request::new(DownloadRequest {
						request_id:uuid::Uuid::new_v4().to_string(),
						url:Reply.download_url.clone(),
						destination_path:String::new(),
						checksum:String::new(),
						headers:std::collections::HashMap::new(),
					});

					match Client.download_update(DownloadReq).await {
						Ok(DownloadResponse) => {
							let Downloaded = DownloadResponse.into_inner();

							if !Downloaded.error.is_empty() {
								dev_log!("update", "error: [UpdateService] Air download failed: {}", Downloaded.error);

								RunTime
									.Run(ShowMessage(
										MessageSeverity::Error,
										format!("Update download failed: {}", Downloaded.error),
										json!(null),
									))
									.await?;
							} else {
								let _ = ApplicationHandle.emit(
									"sky://update/downloaded",
									json!({ "version": Reply.version, "path": Downloaded.file_path }),
								);

								let ApplyReq = tonic::Request::new(ApplyUpdateRequest {
									request_id:uuid::Uuid::new_v4().to_string(),
									version:Reply.version.clone(),
									update_path:Downloaded.file_path.clone(),
								});

								match Client.apply_update(ApplyReq).await {
									Ok(ApplyResponse) => {
										let Applied = ApplyResponse.into_inner();

										if !Applied.error.is_empty() {
											dev_log!(
												"update",
												"error: [UpdateService] Air apply failed: {}",
												Applied.error
											);

											RunTime
												.Run(ShowMessage(
													MessageSeverity::Error,
													format!("Update install failed: {}", Applied.error),
													json!(null),
												))
												.await?;
										} else {
											let _ = ApplicationHandle
												.emit("sky://update/applied", json!({ "version": Reply.version }));

											RunTime
												.Run(ShowMessage(
													MessageSeverity::Info,
													format!(
														"v{} is ready. Restart Mountain to finish the update.",
														Reply.version
													),
													json!(null),
												))
												.await?;
										}
									},

									Err(ApplyStatus) => {
										dev_log!(
											"update",
											"error: [UpdateService] Air apply_update RPC failed: {}",
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
							}
						},

						Err(DownloadStatus) => {
							dev_log!(
								"update",
								"error: [UpdateService] Air download_update RPC failed: {}",
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
