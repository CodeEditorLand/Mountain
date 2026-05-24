//! `AirClient::IndexFiles`

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

		path:String,

		patterns:Vec<String>,

		exclude_patterns:Vec<String>,

		max_depth:u32,
	) -> Result<IndexInfo::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Indexing files in: {}", path);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::IndexRequest;

			let request = IndexRequest { request_id, path, patterns, exclude_patterns, max_depth };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.IndexFiles(Request::new(request)).await {
				Ok(response) => {
					let Response = response.into_inner();

					// Use fields that actually exist in IndexResponse
					dev_log!(
						"grpc",
						"[AirClient] Files indexed: {} (total size: {} bytes)",
						response.files_indexed,
						response.total_size
					);

					Ok(IndexInfo::Struct { files_indexed:response.files_indexed, total_size:response.total_size })
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Index files RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Index files RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
