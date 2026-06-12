//! Forward a call hierarchy incoming request to the registered provider.

use serde_json::json;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;
use ::Vine::Generated::{ProvideCallHierarchyRequest, ProvideCallHierarchyResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideCallHierarchyRequest,
) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing call hierarchy incoming");

	let ItemDTO = json!({
		"name": Request.item.as_ref().map(|I| I.name.as_str()).unwrap_or(""),
		"uri": Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
	});

	let Forward = Service.environment.ProvideCallHierarchyIncomingCalls(ItemDTO);

	let Outcome = match Service.RunCancellable("ProvideCallHierarchyIncomingCalls", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(<ProvideCallHierarchyResponse>::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(<ProvideCallHierarchyResponse>::default())),

		Err(Error) => Err(Status::internal(format!("call hierarchy incoming failed: {}", Error))),
	}
}
