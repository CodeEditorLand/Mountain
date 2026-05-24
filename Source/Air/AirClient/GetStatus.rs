//! `AirClient::GetStatus`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};
use crate::dev_log;

pub fn Fn(This:&Struct, request_id:String) -> Result<AirStatus::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Getting Air daemon status");

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::StatusRequest;

			let request = StatusRequest { request_id };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.GetStatus(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::StatusResponse = response.into_inner();

					dev_log!(
						"grpc",
						"[AirClient] Status retrieved. Active requests: {}",
						response.active_requests
					);

					Ok(AirStatus::Struct {
						version:response.version,
						uptime_seconds:response.uptime_seconds,
						total_requests:response.total_requests,
						successful_requests:response.successful_requests,
						failed_requests:response.failed_requests,
						average_response_time:response.average_response_time,
						memory_usage_mb:response.memory_usage_mb,
						cpu_usage_percent:response.cpu_usage_percent,
						active_requests:response.active_requests,
					})
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get status RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get status RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
