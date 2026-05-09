#![allow(non_snake_case)]

//! Look up a hover from the registered provider. Joins multiple content
//! pieces with a Markdown horizontal-rule separator.

use tonic::{Response, Status};

use url::Url;

use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Position, ProvideHoverRequest, ProvideHoverResponse, Range},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideHoverRequest,
) -> Result<Response<ProvideHoverResponse>, Status> {

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let Position_ = Request.position.as_ref();

	let Line = Position_.map(|P| P.line).unwrap_or(0);

	let Character = Position_.map(|P| P.character).unwrap_or(0);

	dev_log!(
		"provider",

		"ProvideHover entry handle={} uri={} line={} char={}",

		Request.provider_handle,

		URI,

		Line,

		Character
	);

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let PositionDTO_ = PositionDTO { LineNumber:Line, Column:Character };

	match Service.environment.ProvideHover(DocumentURI, PositionDTO_).await {

		Ok(Some(Hover)) => {

			let Markdown = Hover
				.Contents
				.iter()
				.map(|C| C.Value.as_str())
				.collect::<Vec<_>>()
				.join("\n---\n");

			let RangeOption = Hover.Range.map(|R| {
				Range {
					start:Some(Position { line:R.StartLineNumber, character:R.StartColumn }),
					end:Some(Position { line:R.EndLineNumber, character:R.EndColumn }),
				}
			});

			dev_log!(
				"provider",

				"ProvideHover result handle={} contents_len={} hasRange={}",

				Request.provider_handle,

				Markdown.len(),

				RangeOption.is_some()
			);

			Ok(Response::new(ProvideHoverResponse { markdown:Markdown, range:RangeOption }))
		},

		Ok(None) => {

			dev_log!(
				"provider",

				"ProvideHover result handle={} (no provider)",

				Request.provider_handle
			);

			Ok(Response::new(ProvideHoverResponse { markdown:String::new(), range:None }))
		},

		Err(Error) => {

			dev_log!(
				"provider",

				"warn: ProvideHover failed handle={} err={}",

				Request.provider_handle,

				Error
			);

			Err(Status::internal(format!("Hover failed: {}", Error)))
		},
	}
}
