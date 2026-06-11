//! Provides definition locations for a document position by invoking the
//! matching `Definition` provider and deserialising to `Vec<LocationDTO>`.

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::DTO::{LocationDTO::LocationDTO, PositionDTO::PositionDTO, ProviderType::ProviderType},
};
use serde_json::json;
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Vec<LocationDTO>>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Definition)
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
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize Vec<LocationDTO>: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}
