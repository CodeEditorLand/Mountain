//! `DownloadUpdate` Tauri command - hand off update-package
//! download to Air, returning a `DownloadResultDTO::Struct`.

use crate::{
	IPC::WindAirCommands::{DownloadResultDTO, GetAirAddress, GetOrCreateAirClient},
	dev_log,
};

#[tauri::command]
pub async fn DownloadUpdate(
	url:String,

	destination:String,

	checksum:Option<String>,
) -> Result<DownloadResultDTO::Struct, String> {
	dev_log!("grpc", "[WindAirCommands] DownloadUpdate called: {} -> {}", url, destination);

	let air_address = GetAirAddress::Fn()?;

	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let request_id = uuid::Uuid::new_v4().to_string();

	let file_info = client
		.DownloadUpdate(
			request_id,
			url,
			destination,
			checksum.unwrap_or_default(),
			std::collections::HashMap::new(),
		)
		.await
		.map_err(|e| format!("Update download failed: {:?}", e))?;

	let result = DownloadResultDTO::Struct {
		success:true,

		file_path:file_info.file_path,

		file_size:file_info.file_size,

		checksum:file_info.checksum,
	};

	dev_log!(
		"grpc",
		"[WindAirCommands] Update download completed: success={}",
		result.success
	);

	Ok(result)
}
