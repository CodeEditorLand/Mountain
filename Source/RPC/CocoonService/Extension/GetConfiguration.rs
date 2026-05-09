#![allow(non_snake_case)]

//! Look up a workspace configuration value for the requesting extension.
//! Composes `section.key` when both are present, otherwise falls back to
//! whichever side is non-empty.

use tonic::{Response, Status};
use CommonLibrary::Configuration::{
	ConfigurationProvider::ConfigurationProvider,
	DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{GetConfigurationRequest, GetConfigurationResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:GetConfigurationRequest,
) -> Result<Response<GetConfigurationResponse>, Status> {
	let Key = if Request.section.is_empty() {
		if Request.key.is_empty() { None } else { Some(Request.key.clone()) }
	} else if Request.key.is_empty() {
		Some(Request.section.clone())
	} else {
		Some(format!("{}.{}", Request.section, Request.key))
	};

	dev_log!("cocoon", "[CocoonService] get_configuration: key={:?}", Key);

	match Service
		.environment
		.GetConfigurationValue(Key, ConfigurationOverridesDTO::default())
		.await
	{
		Ok(Value) => {
			let Bytes = serde_json::to_vec(&Value).unwrap_or_default();

			Ok(Response::new(GetConfigurationResponse { value:Bytes }))
		},

		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] get_configuration failed: {}", Error);

			Ok(Response::new(GetConfigurationResponse::default()))
		},
	}
}
