//! Forward a document-range-formatting request to the registered provider.

use serde_json::json;
use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideDocumentRangeFormattingRequest, ProvideDocumentRangeFormattingResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideDocumentRangeFormattingRequest,
) -> Result<Response<ProvideDocumentRangeFormattingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document range formatting");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let R = Request.range.as_ref();

	let RangeDTO = json!({
		"StartLineNumber": R.and_then(|R| R.Start.as_ref()).map(|P| P.line).unwrap_or(0),
		"StartColumn": R.and_then(|R| R.Start.as_ref()).map(|P| P.character).unwrap_or(0),
		"EndLineNumber": R.and_then(|R| R.end.as_ref()).map(|P| P.line).unwrap_or(0),
		"EndColumn": R.and_then(|R| R.end.as_ref()).map(|P| P.character).unwrap_or(0),
	});

	let OptionsDTO = json!({ "tabSize": 4, "insertSpaces": true });

	match Service
		.environment
		.ProvideDocumentRangeFormattingEdits(DocumentURI, RangeDTO, OptionsDTO)
		.await
	{
		Ok(_) => Ok(Response::new(ProvideDocumentRangeFormattingResponse::default())),

		Err(Error) => Err(Status::internal(format!("Document range formatting failed: {}", Error))),
	}
}
