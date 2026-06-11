//! Provides on-type formatting edits (triggered by a typed character) by
//! invoking the matching `OnTypeFormatting` provider and deserialising to
//! `Vec<TextEditDTO>`.

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::DTO::{
		PositionDTO::PositionDTO,
		ProviderType::ProviderType,
		TextEditDTO::TextEditDTO,
	},
};
use serde_json::{Value, json};
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	character:String,

	options_dto:Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::OnTypeFormatting)
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
					json!(character),
					options_dto,
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize Vec<TextEditDTO>: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}
