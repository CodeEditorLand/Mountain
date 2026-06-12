//! Forward a workspace-symbols query to the registered provider.
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;
use ::Vine::Generated::{ProvideWorkspaceSymbolsRequest, ProvideWorkspaceSymbolsResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideWorkspaceSymbolsRequest,
) -> Result<Response<ProvideWorkspaceSymbolsResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing workspace symbols for query: {}",
		Request.query
	);

	let Forward = Service.environment.ProvideWorkspaceSymbols(Request.query);

	let Outcome = match Service.RunCancellable("ProvideWorkspaceSymbols", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideWorkspaceSymbolsResponse::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(ProvideWorkspaceSymbolsResponse::default())),

		Err(Error) => Err(Status::internal(format!("Workspace symbols failed: {}", Error))),
	}
}
