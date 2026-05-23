
//! Delegate the update check to the Air gRPC service. Centralised update
//! management across the Land ecosystem; download path still TODO.

#[cfg(feature = "AirIntegration")]
use std::sync::Arc;

#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::air::{air_service_client::AirServiceClient, air_service_server::UpdateCheckRequest};
#[cfg(feature = "AirIntegration")]
use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, ShowMessage::ShowMessage},
};
#[cfg(feature = "AirIntegration")]
use serde_json::json;
#[cfg(feature = "AirIntegration")]
use tauri::AppHandle;

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
		request_id:RequestID,
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
					// TODO: call Air's download_update endpoint, track progress, install.
					RunTime
						.Run(ShowMessage(
							MessageSeverity::Info,
							"Update download via Air is not yet implemented. Please update manually.".to_string(),
							json!(null),
						))
						.await?;
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
