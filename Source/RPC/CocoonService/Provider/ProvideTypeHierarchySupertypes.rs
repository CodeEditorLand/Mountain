//! Forward a type hierarchy supertypes request to the registered provider.
use serde_json::json;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;
use ::Vine::Generated::{ProvideTypeHierarchyRequest, ProvideTypeHierarchyResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideTypeHierarchyRequest,
) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing type hierarchy supertypes");

	let ItemDTO = json!({
		"name": Request.item.as_ref().map(|I| I.name.as_str()).unwrap_or(""),
		"uri": Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
	});

	let Forward = Service.environment.ProvideTypeHierarchySupertypes(ItemDTO);

	let Outcome = match Service.RunCancellable("ProvideTypeHierarchySupertypes", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(<ProvideTypeHierarchyResponse>::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(<ProvideTypeHierarchyResponse>::default())),

		Err(Error) => Err(Status::internal(format!("type hierarchy supertypes failed: {}", Error))),
	}
}
