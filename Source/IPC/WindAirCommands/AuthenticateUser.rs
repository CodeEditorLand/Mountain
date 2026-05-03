#![allow(non_snake_case)]

//! `AuthenticateUser` Tauri command - delegate sign-in to
//! Air's auth service for the named provider (GitHub / GitLab
//! / Microsoft / etc).

use crate::{
	IPC::WindAirCommands::{AuthResponseDTO, GetAirAddress, GetOrCreateAirClient},
	dev_log,
};

#[tauri::command]
pub async fn AuthenticateUser(
	username:String,
	password:String,
	provider:String,
) -> Result<AuthResponseDTO::Struct, String> {
	dev_log!(
		"grpc",
		"[WindAirCommands] AuthenticateUser called: {} via {}",
		username,
		provider
	);

	let air_address = GetAirAddress::Fn()?;
	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let request_id = uuid::Uuid::new_v4().to_string();

	let token = client
		.authenticate(request_id, username, password, provider)
		.await
		.map_err(|e| format!("Authentication failed: {:?}", e))?;

	let result = AuthResponseDTO::Struct { success:true, token, error:None };

	dev_log!("grpc", "[WindAirCommands] Authentication completed: success={}", result.success);
	Ok(result)
}
