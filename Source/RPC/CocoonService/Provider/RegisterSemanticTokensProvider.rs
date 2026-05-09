#![allow(non_snake_case)]

//! Register a Cocoon-side semantic-tokens provider. Uses the
//! semantic-tokens-specific request shape (carries the legend).

use tonic::{Response, Status};

use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, RegisterSemanticTokensProviderRequest},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:RegisterSemanticTokensProviderRequest,
) -> Result<Response<Empty>, Status> {

	dev_log!("cocoon", "[CocoonService] Registering Semantic Tokens Provider");

	Service.RegisterProvider(
		Request.handle,

		ProviderType::SemanticTokens,

		&Request.language_selector,

		&Request.extension_id,
	);

	Ok(Response::new(Empty {}))
}
