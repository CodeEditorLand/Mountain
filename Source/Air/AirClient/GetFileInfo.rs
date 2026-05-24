//! `AirClient::GetFileInfo`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};
use crate::dev_log;

pub fn Fn(This:&Struct, request_id:String, path:String) -> Result<ExtendedFileInfo::Struct, CommonError> {
		let path_display = path.clone();

		dev_log!("grpc", "[AirClient] Getting file info for: {}", path);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::FileInfoRequest;

			let request = FileInfoRequest { request_id, path };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.GetFileInfo(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::FileInfoResponse = response.into_inner();

					dev_log!(
						"grpc",
						"[AirClient] File info retrieved for: {} (exists: {})",
						path_display,
						response.exists
					);

					Ok(ExtendedFileInfo::Struct {
						exists:response.exists,
						size:response.size,
						mime_type:response.mime_type,
						checksum:response.checksum,
						modified_time:response.modified_time,
					})
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get file info RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get file info RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
