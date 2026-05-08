#![allow(non_snake_case)]

//! Register a Cocoon-side call-hierarchy provider.

use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, RegisterProviderRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:RegisterProviderRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering call-hierarchy provider for '{}' with handle {}",
		Request.language_selector,
		Request.handle
	);

	Service.RegisterProvider(
		Request.handle,
		ProviderType::CallHierarchy,
		&Request.language_selector,
		&Request.extension_id,
	);

	Ok(Response::new(Empty {}))
}
