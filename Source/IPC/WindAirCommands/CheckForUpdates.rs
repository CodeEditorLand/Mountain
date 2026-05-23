//! `CheckForUpdates` Tauri command - delegate the update probe
//! to Air's gRPC service and shape the response into
//! `UpdateInfoDTO::Struct`.

use crate::{
	IPC::WindAirCommands::{GetAirAddress, GetOrCreateAirClient, UpdateInfoDTO},
	dev_log,
};

#[tauri::command]
pub async fn CheckForUpdates(
	current_version:Option<String>,

	channel:Option<String>,
) -> Result<UpdateInfoDTO::Struct, String> {
	dev_log!(
		"grpc",
		"[WindAirCommands] CheckForUpdates called with version: {:?}, channel: {:?}",
		current_version,
		channel
	);

	let air_address = GetAirAddress::Fn()?;

	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let request_id = uuid::Uuid::new_v4().to_string();

	let current_version = current_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

	let channel = channel.unwrap_or_else(|| "stable".to_string());

	let update_info = client
		.check_for_updates(request_id, current_version, channel)
		.await
		.map_err(|e| format!("Update check failed: {:?}", e))?;

	let result = UpdateInfoDTO::Struct {
		update_available:update_info.update_available,

		version:update_info.version,

		download_url:update_info.download_url,

		release_notes:update_info.release_notes,
	};

	dev_log!(
		"grpc",
		"[WindAirCommands] Update check completed: available={}",
		result.update_available
	);

	Ok(result)
}
