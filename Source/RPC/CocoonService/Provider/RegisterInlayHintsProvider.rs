//! Register a Cocoon-side inlay-hints provider.

use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{Empty, RegisterProviderRequest};

pub async fn Fn(Service:&CocoonServiceImpl, Request:RegisterProviderRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering inlay-hints provider for '{}' with handle {}",
		Request.language_selector,
		Request.handle
	);

	Service.RegisterProvider(
		Request.handle,
		ProviderType::InlayHint,
		&Request.language_selector,
		&Request.extension_id,
	);

	Ok(Response::new(Empty {}))
}
