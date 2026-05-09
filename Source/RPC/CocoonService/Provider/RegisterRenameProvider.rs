#![allow(non_snake_case)]

//! Register a Cocoon-side rename provider.

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

		"[CocoonService] Registering rename provider for '{}' with handle {}",

		Request.language_selector,

		Request.handle
	);

	Service.RegisterProvider(
		Request.handle,

		ProviderType::Rename,

		&Request.language_selector,

		&Request.extension_id,
	);

	Ok(Response::new(Empty {}))
}
