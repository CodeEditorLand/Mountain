#![allow(non_snake_case)]

//! `GetAirStatus` Tauri command - call Air's status RPC and
//! its health probe, fold both into an
//! `AirServiceStatusDTO::Struct`.

use crate::{
	IPC::WindAirCommands::{AirServiceStatusDTO, GetAirAddress, GetOrCreateAirClient},
	dev_log,
};

#[tauri::command]
pub async fn GetAirStatus() -> Result<AirServiceStatusDTO::Struct, String> {
	dev_log!("grpc", "[WindAirCommands] GetAirStatus called");

	let air_address = GetAirAddress::Fn()?;

	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let request_id = uuid::Uuid::new_v4().to_string();

	let status = client
		.get_status(request_id)
		.await
		.map_err(|e| format!("Failed to get Air status: {:?}", e))?;

	let healthy = client.health_check().await.unwrap_or(false);

	let result = AirServiceStatusDTO::Struct {
		version:status.version,

		uptime_seconds:status.uptime_seconds,

		total_requests:status.total_requests,

		successful_requests:status.successful_requests,

		failed_requests:status.failed_requests,

		active_requests:status.active_requests,

		healthy,
	};

	dev_log!("grpc", "[WindAirCommands] Air status retrieved: healthy={}", result.healthy);

	Ok(result)
}
