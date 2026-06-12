//! Forward a code-lens request to the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;
use ::Vine::Generated::{ProvideCodeLensesRequest, ProvideCodeLensesResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideCodeLensesRequest,
) -> Result<Response<ProvideCodeLensesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing code lenses");

	let DocumentURI = Url::parse(Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""))
		.map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Forward = Service.environment.ProvideCodeLenses(DocumentURI);

	let Outcome = match Service.RunCancellable("ProvideCodeLenses", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideCodeLensesResponse::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(ProvideCodeLensesResponse::default())),

		Err(Error) => Err(Status::internal(format!("Code lenses failed: {}", Error))),
	}
}
