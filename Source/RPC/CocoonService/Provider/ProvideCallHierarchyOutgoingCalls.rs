#![allow(non_snake_case)]

//! Forward a call hierarchy outgoing request to the registered provider.

use serde_json::json;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideCallHierarchyRequest, ProvideCallHierarchyResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,
	Request:ProvideCallHierarchyRequest,
) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing call hierarchy outgoing");
	let ItemDTO = json!({
		"name": Request.item.as_ref().map(|I| I.name.as_str()).unwrap_or(""),
		"uri": Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
	});
	match Service.environment.ProvideCallHierarchyOutgoingCalls(ItemDTO).await {
		Ok(_) => Ok(Response::new(<ProvideCallHierarchyResponse>::default())),
		Err(Error) => Err(Status::internal(format!("call hierarchy outgoing failed: {}", Error))),
	}
}
