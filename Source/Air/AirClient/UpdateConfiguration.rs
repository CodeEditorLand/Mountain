//! `AirClient::UpdateConfiguration`

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

		updates:HashMap<String, String>,
	) -> Result<(), CommonError> {
		let section_display = section.clone();

		dev_log!(
			"grpc",
			"[AirClient] Updating configuration for section: {} ({} keys)",
			section_display,
			updates.len()
		);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::UpdateConfigurationRequest;

			let request = UpdateConfigurationRequest { request_id, section, updates };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.UpdateConfiguration(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::UpdateConfigurationResponse = response.into_inner();

					if response.success {
						dev_log!(
							"grpc",
							"[AirClient] Configuration updated successfully for section: {}",
							section_display
						);

						Ok(())
					} else {
						dev_log!("grpc", "error: [AirClient] Failed to update configuration: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Update configuration RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Update configuration RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
