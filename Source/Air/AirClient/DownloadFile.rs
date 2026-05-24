//! `AirClient::DownloadFile`

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

		url:String,

		destination_path:String,

		checksum:String,

		headers:HashMap<String, String>,
	) -> Result<FileInfo::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Downloading file from: {}", url);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::DownloadRequest;

			let request = DownloadRequest { request_id, url, destination_path, checksum, headers };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.DownloadFile(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::DownloadResponse = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] File downloaded successfully to: {}", response.file_path);

						Ok(FileInfo::Struct {
							file_path:response.file_path,
							file_size:response.file_size,
							checksum:response.checksum,
						})
					} else {
						dev_log!("grpc", "error: [AirClient] File download failed: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Download file RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Download file RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
