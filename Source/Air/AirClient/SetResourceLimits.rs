//! `AirClient::SetResourceLimits`

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

		memory_limit_mb:u32,

		cpu_limit_percent:u32,

		disk_limit_mb:u32,
	) -> Result<(), CommonError> {
		dev_log!(
			"grpc",
			"[AirClient] Setting resource limits: memory={}MB, cpu={}%, disk={}MB",
			memory_limit_mb,
			cpu_limit_percent,
			disk_limit_mb
		);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::ResourceLimitsRequest;

			let request = ResourceLimitsRequest { request_id, memory_limit_mb, cpu_limit_percent, disk_limit_mb };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.SetResourceLimits(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::ResourceLimitsResponse = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] Resource limits set successfully");

						Ok(())
					} else {
						dev_log!("grpc", "error: [AirClient] Failed to set resource limits: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Set resource limits RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Set resource limits RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
