//! Forward an on-type-formatting request to the registered provider.

use serde_json::json;
use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{ProvideOnTypeFormattingRequest, ProvideOnTypeFormattingResponse};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideOnTypeFormattingRequest,
) -> Result<Response<ProvideOnTypeFormattingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing on-type formatting");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Position_ = Request.position.as_ref();

	let PositionDTO_ = PositionDTO {
		LineNumber:Position_.map(|P| P.line).unwrap_or(0),

		Column:Position_.map(|P| P.character).unwrap_or(0),
	};

	let OptionsDTO = json!({ "tabSize": 4, "insertSpaces": true });

	match Service
		.environment
		.ProvideOnTypeFormattingEdits(DocumentURI, PositionDTO_, Request.character, OptionsDTO)
		.await
	{
		Ok(_) => Ok(Response::new(ProvideOnTypeFormattingResponse::default())),

		Err(Error) => Err(Status::internal(format!("On-type formatting failed: {}", Error))),
	}
}
