//! Resolve "go to definition" via the registered provider, mapping each
//! result location into the gRPC `Location` shape.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use ::Vine::Generated::{Location, Position, ProvideDefinitionRequest, ProvideDefinitionResponse, Range, Uri};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideDefinitionRequest,
) -> Result<Response<ProvideDefinitionResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing definition for provider {}",
		Request.provider_handle
	);

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Position_ = Request.position.as_ref();

	let PositionDTO_ = PositionDTO {
		LineNumber:Position_.map(|P| P.line).unwrap_or(0),

		Column:Position_.map(|P| P.character).unwrap_or(0),
	};

	let Forward = Service.environment.ProvideDefinition(DocumentURI, PositionDTO_);

	let Outcome = match Service.RunCancellable("ProvideDefinition", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideDefinitionResponse { locations:Vec::new() })),
	};

	match Outcome {
		Ok(Some(Locations)) => {
			let Mapped = Locations
				.iter()
				.map(|Loc| {
					Location {
						uri:Some(Uri { value:Loc.Uri.to_string() }),
						range:Some(Range {
							start:Some(Position { line:Loc.Range.StartLineNumber, character:Loc.Range.StartColumn }),
							end:Some(Position { line:Loc.Range.EndLineNumber, character:Loc.Range.EndColumn }),
						}),
					}
				})
				.collect();

			Ok(Response::new(ProvideDefinitionResponse { locations:Mapped }))
		},

		Ok(None) => Ok(Response::new(ProvideDefinitionResponse { locations:Vec::new() })),

		Err(Error) => Err(Status::internal(format!("Definition failed: {}", Error))),
	}
}
