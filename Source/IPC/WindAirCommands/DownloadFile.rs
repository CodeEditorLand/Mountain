//! `DownloadFile` Tauri command - generic URL download routed
//! through Air's download service.

use crate::{
	IPC::WindAirCommands::{DownloadResultDTO, GetAirAddress, GetOrCreateAirClient},
	dev_log,
};

#[tauri::command]
pub async fn Fn(url:String, destination:String) -> Result<DownloadResultDTO::Struct, String> {
	dev_log!("grpc", "[WindAirCommands] DownloadFile called: {} -> {}", url, destination);

	let air_address = GetAirAddress::Fn()?;

	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let RequestId = uuid::Uuid::new_v4().to_string();

	let file_info = client
		.DownloadFile(request_id, url, destination, String::new(), std::collections::HashMap::new())
		.await
		.map_err(|E| format!("File download failed: {:?}", e))?;

	let result = DownloadResultDTO::Struct {
		success:true,

		file_path:file_info.file_path,

		file_size:file_info.file_size,

		checksum:file_info.checksum,
	};

	dev_log!("grpc", "[WindAirCommands] File download completed");

	Ok(result)
}
