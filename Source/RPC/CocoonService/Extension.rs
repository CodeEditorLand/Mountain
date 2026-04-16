#![allow(non_snake_case)]
//! Extension domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: get_extension, get_all_extensions, get_configuration.

use CommonLibrary::Configuration::{
	ConfigurationProvider::ConfigurationProvider,
	DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
};
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::dev_log;
use crate::Vine::Generated::{
	Empty, ExtensionInfo, GetAllExtensionsResponse, GetConfigurationRequest,
	GetConfigurationResponse, GetExtensionRequest, GetExtensionResponse,
};

pub async fn GetExtension(
	Service:&CocoonServiceImpl,
	req:GetExtensionRequest,
) -> Result<Response<GetExtensionResponse>, Status> {
	use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

	dev_log!("cocoon", "[CocoonService] get_extension: {}", req.extension_id);

	let ExtensionOption = Service.environment.GetExtension(req.extension_id.clone()).await.ok().flatten();

	let InfoOption = ExtensionOption.map(|Value| {
		ExtensionInfo {
			id:req.extension_id,
			display_name:Value.get("Name").and_then(|V| V.as_str()).unwrap_or("").to_string(),
			version:Value.get("Version").and_then(|V| V.as_str()).unwrap_or("").to_string(),
			is_active:true, // scanned = considered active for now
			extension_path:Value
				.get("ExtensionLocation")
				.and_then(|V| V.as_str())
				.unwrap_or("")
				.to_string(),
		}
	});

	Ok(Response::new(GetExtensionResponse { extension:InfoOption }))
}

pub async fn GetAllExtensions(
	Service:&CocoonServiceImpl,
	_req:Empty,
) -> Result<Response<GetAllExtensionsResponse>, Status> {
	use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

	let Extensions = Service.environment.GetExtensions().await.unwrap_or_default();

	let ExtensionInfoList = Extensions
		.iter()
		.map(|Value| {
			ExtensionInfo {
				id:Value.get("Identifier").and_then(|V| V.as_str()).unwrap_or("").to_string(),
				display_name:Value.get("Name").and_then(|V| V.as_str()).unwrap_or("").to_string(),
				version:Value.get("Version").and_then(|V| V.as_str()).unwrap_or("").to_string(),
				is_active:true,
				extension_path:Value
					.get("ExtensionLocation")
					.and_then(|V| V.as_str())
					.unwrap_or("")
					.to_string(),
			}
		})
		.collect();

	Ok(Response::new(GetAllExtensionsResponse { extensions:ExtensionInfoList }))
}

pub async fn GetConfiguration(
	Service:&CocoonServiceImpl,
	req:GetConfigurationRequest,
) -> Result<Response<GetConfigurationResponse>, Status> {
	let Key = if req.section.is_empty() {
		if req.key.is_empty() { None } else { Some(req.key.clone()) }
	} else if req.key.is_empty() {
		Some(req.section.clone())
	} else {
		Some(format!("{}.{}", req.section, req.key))
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
