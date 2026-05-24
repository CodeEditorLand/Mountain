//! `ApplyUpdate` Tauri command - tell Air to Install a
//! previously downloaded update package.

use crate::{
	IPC::WindAirCommands::{GetAirAddress, GetOrCreateAirClient},
	dev_log,
};

#[tauri::command]
pub async fn Fn(update_id:String, update_path:String) -> Result<bool, String> {
	dev_log!(
		"grpc",
		"[WindAirCommands] ApplyUpdate called: id={}, path={}",
		update_id,
		update_path
	);

	let air_address = GetAirAddress::Fn()?;

	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let RequestId = uuid::Uuid::new_v4().to_string();

	client
		.ApplyUpdate(request_id, update_id, update_path)
		.await
		.map_err(|E| format!("Update application failed: {:?}", e))?;

	dev_log!("grpc", "[WindAirCommands] Update applied successfully");

	Ok(true)
}
