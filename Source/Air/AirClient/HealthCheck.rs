//! `AirClient::HealthCheck`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};
use crate::dev_log;

pub fn Fn(This:&Struct) -> Result<bool, CommonError> {
		dev_log!("grpc", "[AirClient] Performing health check");

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::HealthCheckRequest;

			let request = HealthCheckRequest {};

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.HealthCheck(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::HealthCheckResponse = response.into_inner();

					dev_log!("grpc", "[AirClient] Health check result: {}", response.healthy);

					Ok(response.healthy)
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Health check RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Health check RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			// When AirIntegration is not enabled, we return true to allow
			// the application to function without Air
			Ok(true)
		}
	}
