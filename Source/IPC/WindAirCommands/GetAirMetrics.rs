
//! `GetAirMetrics` Tauri command - pull memory / CPU / disk /
//! network counters from Air, optionally filtered by metric
//! type ("performance", "resources", "requests").

use crate::{
	IPC::WindAirCommands::{AirMetricsDTO, GetAirAddress, GetOrCreateAirClient},
	dev_log,
};

#[tauri::command]
pub async fn GetAirMetrics(metric_type:Option<String>) -> Result<AirMetricsDTO::Struct, String> {
	dev_log!("grpc", "[WindAirCommands] GetAirMetrics called with type: {:?}", metric_type);

	let air_address = GetAirAddress::Fn()?;

	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let request_id = uuid::Uuid::new_v4().to_string();

	let metrics = client
		.get_metrics(request_id, metric_type)
		.await
		.map_err(|e| format!("Failed to get Air metrics: {:?}", e))?;

	let result = AirMetricsDTO::Struct {
		memory_usage_mb:metrics.memory_usage_mb,

		cpu_usage_percent:metrics.cpu_usage_percent,

		average_response_time:metrics.average_response_time,

		disk_usage_mb:metrics.disk_usage_mb,

		network_usage_mbps:metrics.network_usage_mbps,
	};

	dev_log!("grpc", "[WindAirCommands] Air metrics retrieved");

	Ok(result)
}
