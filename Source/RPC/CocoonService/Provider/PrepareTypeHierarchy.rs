#![allow(non_snake_case)]

//! `PrepareTypeHierarchy` gRPC RPC handler.
//!
//! Entry point for VS Code's type hierarchy feature. Returns the root
//! `TypeHierarchyItem` at the given position so the Subtypes/Supertypes
//! panels have a starting item to display.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideTypeHierarchyRequest, ProvideTypeHierarchyResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,
	Request:ProvideTypeHierarchyRequest,
) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let Position_ = Request.position.as_ref();
	let Line = Position_.map(|P| P.line).unwrap_or(0);
	let Character = Position_.map(|P| P.character).unwrap_or(0);

	dev_log!(
		"provider",
		"PrepareTypeHierarchy handle={} uri={} line={} char={}",
		Request.provider_handle,
		URI,
		Line,
		Character
	);

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;
	let PositionDTO_ = PositionDTO { LineNumber:Line, Column:Character };

	match Service.environment.PrepareTypeHierarchy(DocumentURI, PositionDTO_).await {
		Ok(_) => Ok(Response::new(ProvideTypeHierarchyResponse::default())),
		Err(Error) => Err(Status::internal(format!("prepare type hierarchy failed: {}", Error))),
	}
}
