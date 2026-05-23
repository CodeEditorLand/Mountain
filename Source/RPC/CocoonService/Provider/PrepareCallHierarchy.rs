
//! `PrepareCallHierarchy` gRPC RPC handler.
//!
//! The entry-point call for VS Code's call hierarchy feature. Mountain calls
//! this with `uri + position`; Cocoon's `$prepareCallHierarchyItems` dispatch
//! asks the registered provider to return the root `CallHierarchyItem` at that
//! location. Without this step the incoming/outgoing calls panels are always
//! empty even when the provider is correctly registered.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideCallHierarchyRequest, ProvideCallHierarchyResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideCallHierarchyRequest,
) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let Position_ = Request.position.as_ref();

	let Line = Position_.map(|P| P.line).unwrap_or(0);

	let Character = Position_.map(|P| P.character).unwrap_or(0);

	dev_log!(
		"provider",
		"PrepareCallHierarchy handle={} uri={} line={} char={}",
		Request.provider_handle,
		URI,
		Line,
		Character
	);

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let PositionDTO_ = PositionDTO { LineNumber:Line, Column:Character };

	match Service.environment.PrepareCallHierarchy(DocumentURI, PositionDTO_).await {
		Ok(_) => Ok(Response::new(ProvideCallHierarchyResponse::default())),

		Err(Error) => Err(Status::internal(format!("prepare call hierarchy failed: {}", Error))),
	}
}
