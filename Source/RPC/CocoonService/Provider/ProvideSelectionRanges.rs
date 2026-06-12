//! Forward a selection-ranges request (multiple positions per call) to
//! the registered provider.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use ::Vine::Generated::{ProvideSelectionRangesRequest, ProvideSelectionRangesResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideSelectionRangesRequest,
) -> Result<Response<ProvideSelectionRangesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing selection ranges");

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let PositionDTOs:Vec<PositionDTO> = Request
		.positions
		.iter()
		.map(|P| PositionDTO { LineNumber:P.line, Column:P.character })
		.collect();

	let Forward = Service.environment.ProvideSelectionRanges(DocumentURI, PositionDTOs);

	let Outcome = match Service.RunCancellable("ProvideSelectionRanges", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideSelectionRangesResponse::default())),
	};

	match Outcome {
		Ok(_) => Ok(Response::new(ProvideSelectionRangesResponse::default())),

		Err(Error) => Err(Status::internal(format!("Selection ranges failed: {}", Error))),
	}
}
