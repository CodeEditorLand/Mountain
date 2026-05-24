//! `AirClient::Authenticate`

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

		username:String,

		password:String,

		provider:String,
	) -> Result<String, CommonError> {
		dev_log!(
			"grpc",
			"[AirClient] Authenticating user '{}' with provider '{}'",
			username,
			provider
		);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::AuthenticationRequest;

			let username_display = username.clone();

			let request = AuthenticationRequest { request_id, username, password, provider };

			let client = self
				.Client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.Authenticate(Request::new(request)).await {
				Ok(response) => {
					let Response = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] Authentication successful for user '{}'", username_display);

						Ok(response.token)
					} else {
						dev_log!(
							"grpc",
							"error: [AirClient] Authentication failed for user '{}': {}",
							username_display,
							response.error
						);

						Err(CommonError::AccessDenied { Reason:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Authentication RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Authentication RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
