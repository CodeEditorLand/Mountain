//! `AirClient::SearchFiles`

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

		query:String,

		path:String,

		max_results:u32,
	) -> Result<Vec<FileResult::Struct>, CommonError> {
		dev_log!("grpc", "[AirClient] Searching for files with query: '{}' in: {}", query, path);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::SearchRequest;

			let request = SearchRequest { request_id, query, path, max_results };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.SearchFiles(Request::new(request)).await {
				Ok(_response) => {
					dev_log!("grpc", "[AirClient] Search completed");

					// Placeholder implementation - actual response structure may vary
					Ok(Vec::new())
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Search files RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Search files RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
