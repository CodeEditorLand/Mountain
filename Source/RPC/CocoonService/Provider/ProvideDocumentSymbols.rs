//! Forward a document-symbols request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;
use ::Vine::Generated::{ProvideDocumentSymbolsRequest, ProvideDocumentSymbolsResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideDocumentSymbolsRequest,
) -> Result<Response<ProvideDocumentSymbolsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document symbols");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Forward = Service.environment.ProvideDocumentSymbols(DocumentURI);

	let Outcome = match Service.RunCancellable("ProvideDocumentSymbols", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideDocumentSymbolsResponse::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(ProvideDocumentSymbolsResponse::default())),

		Err(Error) => Err(Status::internal(format!("Document symbols failed: {}", Error))),
	}
}
