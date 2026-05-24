//! `AirClient::ApplyUpdate`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};
use crate::dev_log;

pub fn Fn(This:&Struct, request_id:String, version:String, update_path:String) -> Result<(), CommonError> {
		dev_log!("grpc", "[AirClient] Applying update version: {}", version);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::ApplyUpdateRequest;

			let request = ApplyUpdateRequest { request_id, version, update_path };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.ApplyUpdate(Request::new(request)).await {
				Ok(response) => {
					let Response:AirLibrary::Vine::Generated::air::ApplyUpdateResponse = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] Update applied successfully");

						Ok(())
					} else {
						dev_log!("grpc", "error: [AirClient] Update application failed: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Apply update RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Apply update RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
