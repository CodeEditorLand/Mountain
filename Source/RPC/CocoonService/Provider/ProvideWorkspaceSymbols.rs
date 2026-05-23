//! Forward a workspace-symbols query to the registered provider.

use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideWorkspaceSymbolsRequest, ProvideWorkspaceSymbolsResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideWorkspaceSymbolsRequest,
) -> Result<Response<ProvideWorkspaceSymbolsResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing workspace symbols for query: {}",
		Request.query
	);

	match Service.environment.ProvideWorkspaceSymbols(Request.query).await {
		Ok(_) => Ok(Response::new(ProvideWorkspaceSymbolsResponse::default())),

		Err(Error) => Err(Status::internal(format!("Workspace symbols failed: {}", Error))),
	}
}
