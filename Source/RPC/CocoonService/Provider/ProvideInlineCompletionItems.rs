//! `ProvideInlineCompletionItems` gRPC RPC handler.
//!
//! Called by Mountain's `IInlineCompletionsProvider` (wired into Monaco via
//! the language feature registry) when the cursor pauses or the user presses
//! a completion trigger key. Forwards the request to Cocoon which dispatches
//! `$provideInlineCompletionItems` to the registered extension provider
//! (GitHub Copilot, Roo Code, Continue, etc.).
//!
//! The response is a list of `InlineCompletionItem` messages that Mountain
//! maps back into Monaco's `InlineCompletionList`.

use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use serde_json::json;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{InlineCompletionItem, ProvideInlineCompletionRequest, ProvideInlineCompletionResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideInlineCompletionRequest,
) -> Result<Response<ProvideInlineCompletionResponse>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let Position_ = Request.position.as_ref();

	let Line = Position_.map(|P| P.line).unwrap_or(0);

	let Character = Position_.map(|P| P.character).unwrap_or(0);

	dev_log!(
		"provider",
		"ProvideInlineCompletionItems handle={} uri={} line={} char={}",
		Request.provider_handle,
		URI,
		Line,
		Character
	);

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let PositionDTO_ = PositionDTO { LineNumber:Line, Column:Character };

	let Context = json!({
		"triggerKind": Request.context.as_ref().map(|C| C.trigger_kind).unwrap_or(0),
		"selectedCompletionInfo": Request.context.as_ref()
			.map(|C| C.selected_completion_info.as_str())
			.unwrap_or(""),
	});

	match Service
		.environment
		.ProvideInlineCompletionItems(DocumentURI, PositionDTO_, Context)
		.await
	{
		Ok(Some(Raw)) => {
			// Raw is a JSON Value returned by Cocoon's provider.
			// Shape: { items: [{ insertText, range?, isSnippet?, command? }] }
			// or an array directly.
			let ItemsArr = Raw
				.get("items")
				.and_then(|V| V.as_array())
				.cloned()
				.or_else(|| Raw.as_array().cloned())
				.unwrap_or_default();

			let Items:Vec<InlineCompletionItem> = ItemsArr
				.iter()
				.filter_map(|Item| {
					let InsertText = Item
						.get("insertText")
						.and_then(|V| V.as_str())
						.or_else(|| Item.get("text").and_then(|V| V.as_str()))
						.unwrap_or("");

					if InsertText.is_empty() {
						return None;
					}

					let IsSnippet = Item.get("isSnippet").and_then(|V| V.as_bool()).unwrap_or(false);

					let Command = Item
						.get("command")
						.and_then(|V| V.get("command"))
						.and_then(|V| V.as_str())
						.or_else(|| Item.get("command").and_then(|V| V.as_str()))
						.unwrap_or("")
						.to_string();

					// prost generates snake_case field names from proto PascalCase.
					Some(InlineCompletionItem {
						insert_text:InsertText.to_string(),
						range:None, // Range hydration deferred to Mountain's provider impl
						command:Command,
						is_snippet:IsSnippet,
					})
				})
				.collect();

			Ok(Response::new(ProvideInlineCompletionResponse { items:Items }))
		},

		Ok(None) => Ok(Response::new(ProvideInlineCompletionResponse::default())),

		Err(Error) => {
			dev_log!("provider", "warn: [ProvideInlineCompletionItems] provider error: {}", Error);
			Ok(Response::new(ProvideInlineCompletionResponse::default()))
		},
	}
}
