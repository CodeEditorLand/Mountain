//! Forward a rename-edits request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use ::Vine::Generated::{ProvideRenameEditsRequest, ProvideRenameEditsResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideRenameEditsRequest,
) -> Result<Response<ProvideRenameEditsResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing rename edits: new_name={}",
		Request.new_name
	);

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Position_ = Request.position.as_ref();

	let PositionDTO_ = PositionDTO {
		LineNumber:Position_.map(|P| P.line).unwrap_or(0),

		Column:Position_.map(|P| P.character).unwrap_or(0),
	};

	let Forward = Service.environment.ProvideRenameEdits(DocumentURI, PositionDTO_, Request.new_name);

	let Outcome = match Service.RunCancellable("ProvideRenameEdits", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideRenameEditsResponse::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(ProvideRenameEditsResponse::default())),

		Err(Error) => Err(Status::internal(format!("Rename edits failed: {}", Error))),
	}
}
