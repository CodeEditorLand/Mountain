#![allow(non_snake_case)]

//! Forward a document-symbols request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideDocumentSymbolsRequest, ProvideDocumentSymbolsResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideDocumentSymbolsRequest,
) -> Result<Response<ProvideDocumentSymbolsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document symbols");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	match Service.environment.ProvideDocumentSymbols(DocumentURI).await {
		Ok(_) => Ok(Response::new(ProvideDocumentSymbolsResponse::default())),

		Err(Error) => Err(Status::internal(format!("Document symbols failed: {}", Error))),
	}
}
