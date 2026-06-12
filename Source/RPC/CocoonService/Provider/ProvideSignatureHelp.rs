//! Forward a signature-help request to the registered provider.

use serde_json::json;
use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use ::Vine::Generated::{ProvideSignatureHelpRequest, ProvideSignatureHelpResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideSignatureHelpRequest,
) -> Result<Response<ProvideSignatureHelpResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing signature help");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Position_ = Request.position.as_ref();

	let PositionDTO_ = PositionDTO {
		LineNumber:Position_.map(|P| P.line).unwrap_or(0),

		Column:Position_.map(|P| P.character).unwrap_or(0),
	};

	let ContextDTO = json!({ "triggerKind": 1, "isRetrigger": false });

	let Forward = Service.environment.ProvideSignatureHelp(DocumentURI, PositionDTO_, ContextDTO);

	let Outcome = match Service.RunCancellable("ProvideSignatureHelp", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideSignatureHelpResponse::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(ProvideSignatureHelpResponse::default())),

		Err(Error) => Err(Status::internal(format!("Signature help failed: {}", Error))),
	}
}
