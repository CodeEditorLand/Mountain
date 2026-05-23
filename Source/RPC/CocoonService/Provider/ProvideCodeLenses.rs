
//! Forward a code-lens request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideCodeLensesRequest, ProvideCodeLensesResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideCodeLensesRequest,
) -> Result<Response<ProvideCodeLensesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing code lenses");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	match Service.environment.ProvideCodeLenses(DocumentURI).await {
		Ok(_) => Ok(Response::new(ProvideCodeLensesResponse::default())),

		Err(Error) => Err(Status::internal(format!("Code lenses failed: {}", Error))),
	}
}
