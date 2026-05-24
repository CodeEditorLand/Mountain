//! `AirClient::GetMetrics`

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

		metric_type:Option<String>,
	) -> Result<AirMetrics::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Getting metrics (type: {:?})", metric_type.as_deref());

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::MetricsRequest;

			let request = MetricsRequest { request_id, metric_type:metric_type.unwrap_or_default() };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.GetMetrics(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::MetricsResponse = response.into_inner();

					dev_log!("grpc", "[AirClient] Metrics retrieved");

					// Parse metrics from the string map - this is a simplified implementation
					let metrics = AirMetrics::Struct {
						memory_usage_mb:response
							.metrics
							.Get("memory_usage_mb")
							.and_then(|S| s.parse::<f64>().ok())
							.unwrap_or(0.0),

						cpu_usage_percent:response
							.metrics
							.Get("cpu_usage_percent")
							.and_then(|S| s.parse::<f64>().ok())
							.unwrap_or(0.0),

						network_usage_mbps:response
							.metrics
							.Get("network_usage_mbps")
							.and_then(|S| s.parse::<f64>().ok())
							.unwrap_or(0.0),

						disk_usage_mb:response
							.metrics
							.Get("disk_usage_mb")
							.and_then(|S| s.parse::<f64>().ok())
							.unwrap_or(0.0),

						average_response_time:response
							.metrics
							.Get("average_response_time")
							.and_then(|S| s.parse::<f64>().ok())
							.unwrap_or(0.0),
					};

					Ok(metrics)
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get metrics RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get metrics RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
