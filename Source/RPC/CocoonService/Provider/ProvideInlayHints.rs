//! Forward an inlay-hints request to the registered provider.

use serde_json::json;
use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{ProvideInlayHintsRequest, ProvideInlayHintsResponse};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideInlayHintsRequest,
) -> Result<Response<ProvideInlayHintsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing inlay hints");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let R = Request.range.as_ref();

	let RangeDTO = json!({
		"StartLineNumber": R.and_then(|R| R.start.as_ref()).map(|P| P.line).unwrap_or(0),
		"StartColumn": R.and_then(|R| R.start.as_ref()).map(|P| P.character).unwrap_or(0),
		"EndLineNumber": R.and_then(|R| R.end.as_ref()).map(|P| P.line).unwrap_or(0),
		"EndColumn": R.and_then(|R| R.end.as_ref()).map(|P| P.character).unwrap_or(0),
	});

	match Service.environment.ProvideInlayHints(DocumentURI, RangeDTO).await {
		Ok(_) => Ok(Response::new(ProvideInlayHintsResponse::default())),

		Err(Error) => Err(Status::internal(format!("Inlay hints failed: {}", Error))),
	}
}
