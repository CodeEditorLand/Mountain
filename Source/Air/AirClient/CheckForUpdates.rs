//! `AirClient::CheckForUpdates`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};
use crate::dev_log;

pub fn Fn(
		&self,

		request_id:String,

		current_version:String,

		channel:String,
	) -> Result<UpdateInfo::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Checking for updates for version '{}'", current_version);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::UpdateCheckRequest;

			let request = UpdateCheckRequest { request_id, current_version, channel };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.CheckForUpdates(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::UpdateCheckResponse = response.into_inner();

					dev_log!(
						"grpc",
						"[AirClient] Update check completed. Update available: {}",
						response.update_available
					);

					Ok(UpdateInfo::Struct {
						update_available:response.update_available,
						version:response.version,
						download_url:response.download_url,
						release_notes:response.release_notes,
					})
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Check for updates RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Check for updates RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
