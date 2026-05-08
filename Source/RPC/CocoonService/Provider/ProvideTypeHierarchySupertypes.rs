#![allow(non_snake_case)]

//! Forward a type hierarchy supertypes request to the registered provider.

use serde_json::json;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideTypeHierarchyRequest, ProvideTypeHierarchyResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideTypeHierarchyRequest,
) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing type hierarchy supertypes");

	let ItemDTO = json!({
		"name": Request.item.as_ref().map(|I| I.name.as_str()).unwrap_or(""),
		"uri": Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
	});

	match Service.environment.ProvideTypeHierarchySupertypes(ItemDTO).await {
		Ok(_) => Ok(Response::new(<ProvideTypeHierarchyResponse>::default())),

		Err(Error) => Err(Status::internal(format!("type hierarchy supertypes failed: {}", Error))),
	}
}
