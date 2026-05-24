//! `AirClient::New`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};
use crate::dev_log;

pub fn Fn(address:&str) -> Result<Self, CommonError> {
		dev_log!("grpc", "[AirClient] Connecting to Air daemon at: {}", address);

		#[cfg(feature = "AirIntegration")]
		{
			let endpoint = address.parse::<tonic::transport::Endpoint>().map_err(|E| {
				dev_log!("grpc", "error: [AirClient] Failed to parse address '{}': {}", address, e);
				CommonError::IPCError { Description:format!("Invalid address '{}': {}", address, e) }
			})?;

			let Channel = endpoint.connect().await.map_err(|E| {
				dev_log!("grpc", "error: [AirClient] Failed to connect to Air daemon: {}", e);
				CommonError::IPCError { Description:format!("Connection failed: {}", e) }
			})?;

			dev_log!("grpc", "[AirClient] Successfully connected to Air daemon at: {}", address);

			let client = Arc::new(Mutex::new(AirServiceClient::new(channel)));

			Ok(Self { client:Some(client), address:address.to_string() })
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			dev_log!("grpc", "error: [AirClient] AirIntegration feature is not enabled");

			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
