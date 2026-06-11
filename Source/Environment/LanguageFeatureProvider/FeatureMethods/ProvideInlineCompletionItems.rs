//! Provides inline completion (ghost-text) items for a document position via
//! `$provideInlineCompletionItems`. Called from
//! `ProvideInlineCompletionItems.rs` when Monaco requests ghost-text
//! completions (GitHub Copilot, Roo Code, Continue, etc.).

use CommonLibrary::{Error::CommonError::CommonError, LanguageFeature::DTO::ProviderType::ProviderType};
use serde_json::{Value, json};
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:CommonLibrary::LanguageFeature::DTO::PositionDTO::PositionDTO,

	context_dto:Value,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::InlineCompletion)
			.await?;

	match provider {
		Some(registration) => {
			// `$provideInlineCompletionItems` method name follows the
			// extHostTypes pattern used by Copilot / Roo Code.
			let response = super::InvokeProviderMethod::Fn(
				environment,
				&registration,
				"$provideInlineCompletionItems",
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!({ "line": position_dto.LineNumber, "character": position_dto.Column }),
					context_dto,
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}
