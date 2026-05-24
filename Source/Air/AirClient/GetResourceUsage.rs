//! `AirClient::GetResourceUsage`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};
use crate::dev_log;

pub fn Fn(This:&Struct, request_id:String) -> Result<ResourceUsage::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Getting resource usage");

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::ResourceUsageRequest;

			let request = ResourceUsageRequest { request_id };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.GetResourceUsage(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::ResourceUsageResponse = response.into_inner();

					dev_log!("grpc", "[AirClient] Resource usage retrieved");

					Ok(ResourceUsage::Struct {
						memory_usage_mb:response.memory_usage_mb,
						cpu_usage_percent:response.cpu_usage_percent,
						disk_usage_mb:response.disk_usage_mb,
						network_usage_mbps:response.network_usage_mbps,
						thread_count:0,      // Not provided in ResourceUsageResponse
						open_file_handles:0, // Not provided in ResourceUsageResponse
					})
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get resource usage RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get resource usage RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
