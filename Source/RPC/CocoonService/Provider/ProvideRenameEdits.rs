#![allow(non_snake_case)]

//! Forward a rename-edits request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideRenameEditsRequest, ProvideRenameEditsResponse},
	dev_log,
};

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
	match Service
		.environment
		.ProvideRenameEdits(DocumentURI, PositionDTO_, Request.new_name)
		.await
	{
		Ok(_) => Ok(Response::new(ProvideRenameEditsResponse::default())),
		Err(Error) => Err(Status::internal(format!("Rename edits failed: {}", Error))),
	}
}
