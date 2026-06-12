//! Forward a document-highlight request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use ::Vine::Generated::{ProvideDocumentHighlightsRequest, ProvideDocumentHighlightsResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideDocumentHighlightsRequest,
) -> Result<Response<ProvideDocumentHighlightsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document highlights");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Position_ = Request.position.as_ref();

	let PositionDTO_ = PositionDTO {
		LineNumber:Position_.map(|P| P.line).unwrap_or(0),

		Column:Position_.map(|P| P.character).unwrap_or(0),
	};

	let Forward = Service.environment.ProvideDocumentHighlights(DocumentURI, PositionDTO_);

	let Outcome = match Service.RunCancellable("ProvideDocumentHighlights", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideDocumentHighlightsResponse::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(ProvideDocumentHighlightsResponse::default())),

		Err(Error) => Err(Status::internal(format!("Document highlights failed: {}", Error))),
	}
}
