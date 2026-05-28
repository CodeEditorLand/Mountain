//! Forward a document-formatting request to the registered provider.

use serde_json::json;
use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;
use ::Vine::Generated::{ProvideDocumentFormattingRequest, ProvideDocumentFormattingResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideDocumentFormattingRequest,
) -> Result<Response<ProvideDocumentFormattingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document formatting");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let OptionsDTO = json!({ "tabSize": 4, "insertSpaces": true });

	match Service
		.environment
		.ProvideDocumentFormattingEdits(DocumentURI, OptionsDTO)
		.await
	{
		Ok(_) => Ok(Response::new(ProvideDocumentFormattingResponse::default())),

		Err(Error) => Err(Status::internal(format!("Document formatting failed: {}", Error))),
	}
}
