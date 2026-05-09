#![allow(non_snake_case)]

//! Forward a semantic-tokens-full request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideSemanticTokensRequest, ProvideSemanticTokensResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideSemanticTokensRequest,
) -> Result<Response<ProvideSemanticTokensResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing semantic tokens");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	match Service.environment.ProvideSemanticTokensFull(DocumentURI).await {
		Ok(_) => Ok(Response::new(ProvideSemanticTokensResponse::default())),

		Err(Error) => Err(Status::internal(format!("Semantic tokens failed: {}", Error))),
	}
}
