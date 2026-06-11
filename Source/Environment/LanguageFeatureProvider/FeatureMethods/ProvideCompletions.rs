//! Provides completion items at a document position by invoking the matching
//! `Completion` provider and deserialising the result to `CompletionListDTO`.

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::DTO::{
		CompletionContextDTO::CompletionContextDTO,
		CompletionListDTO::CompletionListDTO,
		PositionDTO::PositionDTO,
		ProviderType::ProviderType,
	},
};
use serde_json::{Value, json};
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	context_dto:CompletionContextDTO,

	cancellation_token_value:Option<Value>,
) -> Result<Option<CompletionListDTO>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Completion)
			.await?;

	match provider {
		Some(registration) => {
			let response = super::InvokeProvider::Fn(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
					json!(context_dto),
					cancellation_token_value.unwrap_or_else(|| json!(null)),
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize CompletionListDTO: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}
