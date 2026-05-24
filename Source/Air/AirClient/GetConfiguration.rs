//! `AirClient::GetConfiguration`

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

		section:String,
	) -> Result<HashMap<String, String>, CommonError> {
		let section_display = section.clone();

		dev_log!("grpc", "[AirClient] Getting configuration for section: {}", section);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::ConfigurationRequest;

			let request = ConfigurationRequest { request_id, section };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.GetConfiguration(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::ConfigurationResponse = response.into_inner();

					dev_log!(
						"grpc",
						"[AirClient] Configuration retrieved for section: {} ({} keys)",
						section_display,
						response.configuration.len()
					);

					Ok(response.configuration)
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get configuration RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get configuration RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
