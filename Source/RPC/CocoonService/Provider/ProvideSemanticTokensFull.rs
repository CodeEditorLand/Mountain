//! Forward a semantic-tokens-full request to the registered provider.
use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;
use ::Vine::Generated::{ProvideSemanticTokensRequest, ProvideSemanticTokensResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideSemanticTokensRequest,
) -> Result<Response<ProvideSemanticTokensResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing semantic tokens");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Forward = Service.environment.ProvideSemanticTokensFull(DocumentURI);

	let Outcome = match Service.RunCancellable("ProvideSemanticTokensFull", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideSemanticTokensResponse::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(ProvideSemanticTokensResponse::default())),

		Err(Error) => Err(Status::internal(format!("Semantic tokens failed: {}", Error))),
	}
}
