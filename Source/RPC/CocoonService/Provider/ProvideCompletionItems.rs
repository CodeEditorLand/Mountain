//! Forward a completion request to the registered provider and project
//! the suggestions into the gRPC `CompletionItem` shape.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::{
		CompletionContextDTO::{CompletionContextDTO, CompletionTriggerKindDTO},
		PositionDTO::PositionDTO,
	},
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use ::Vine::Generated::{CompletionItem, ProvideCompletionItemsRequest, ProvideCompletionItemsResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideCompletionItemsRequest,
) -> Result<Response<ProvideCompletionItemsResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing completions for provider {}",
		Request.provider_handle
	);

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let Position_ = Request.position.as_ref();

	let PositionDTO_ = PositionDTO {
		LineNumber:Position_.map(|P| P.line).unwrap_or(0),

		Column:Position_.map(|P| P.character).unwrap_or(0),
	};

	let ContextDTO = CompletionContextDTO {
		TriggerKind:CompletionTriggerKindDTO::Invoke,

		TriggerCharacter:if Request.trigger_character.is_empty() {
			None
		} else {
			Some(Request.trigger_character.clone())
		},
	};

	match Service
		.environment
		.ProvideCompletions(DocumentURI, PositionDTO_, ContextDTO, None)
		.await
	{
		Ok(Some(List)) => {
			let Items = List
				.Suggestions
				.iter()
				.map(|S| {
					CompletionItem {
						label:S.Label.as_str().map(|L| L.to_string()).unwrap_or_default(),
						kind:format!("{}", S.Kind),
						detail:S.Detail.clone().unwrap_or_default(),
						documentation:Vec::new(),
						insert_text:S.InsertText.as_ref().and_then(|V| V.as_str()).unwrap_or("").to_string(),
					}
				})
				.collect();

			Ok(Response::new(ProvideCompletionItemsResponse { items:Items }))
		},

		Ok(None) => Ok(Response::new(ProvideCompletionItemsResponse { items:Vec::new() })),

		Err(Error) => Err(Status::internal(format!("Completions failed: {}", Error))),
	}
}
