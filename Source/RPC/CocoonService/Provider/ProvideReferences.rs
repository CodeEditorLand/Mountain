//! Resolve "find references" via the registered provider, mapping each
//! result into the gRPC `Location` shape.

use serde_json::json;
use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Location, Position, ProvideReferencesRequest, ProvideReferencesResponse, Range, Uri},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideReferencesRequest,
) -> Result<Response<ProvideReferencesResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing references for provider {}",
		Request.ProviderHandle
	);

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Position_ = Request.position.as_ref();

	let PositionDTO_ = PositionDTO {
		LineNumber:Position_.map(|P| P.line).unwrap_or(0),

		Column:Position_.map(|P| P.character).unwrap_or(0),
	};

	let ContextDTO = json!({ "includeDeclaration": true });

	match Service
		.environment
		.ProvideReferences(DocumentURI, PositionDTO_, ContextDTO)
		.await
	{
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

			Ok(Response::new(ProvideReferencesResponse { locations:Mapped }))
		},

		Ok(None) => Ok(Response::new(ProvideReferencesResponse { locations:Vec::new() })),

		Err(Error) => Err(Status::internal(format!("References failed: {}", Error))),
	}
}
