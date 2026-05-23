//! Health probe for the Air daemon. `true` when the gRPC `health_check`
//! responds and reports `healthy`.

#[cfg(feature = "AirIntegration")]
use std::sync::Arc;

#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::air::{air_service_client::AirServiceClient, air_service_server::HealthCheckRequest};

#[cfg(feature = "AirIntegration")]
use crate::dev_log;

#[cfg(feature = "AirIntegration")]
pub async fn Fn(AirClient:&AirServiceClient<tonic::transport::Channel>) -> bool {
	use tonic::Request;

	match AirClient.health_check(Request::new(HealthCheckRequest {})).await {
		Ok(Response) => {
			let IsHealthy = Response.into_inner().healthy;

			if !IsHealthy {
				dev_log!("update", "warn: [UpdateService] Air health check returned unhealthy");
			}

			IsHealthy
		},

		Err(Error) => {
			dev_log!("update", "warn: [UpdateService] Air health check failed: {}", Error);

			false
		},
	}
}
