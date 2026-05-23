
//! Forward a folding-ranges request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideFoldingRangesRequest, ProvideFoldingRangesResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideFoldingRangesRequest,
) -> Result<Response<ProvideFoldingRangesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing folding ranges");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	match Service.environment.ProvideFoldingRanges(DocumentURI).await {
		Ok(_) => Ok(Response::new(ProvideFoldingRangesResponse::default())),

		Err(Error) => Err(Status::internal(format!("Folding ranges failed: {}", Error))),
	}
}
