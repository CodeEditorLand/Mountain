//! Provides workspace edits for renaming a symbol by invoking the matching
//! `Rename` provider in its owning sidecar.

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::DTO::{PositionDTO::PositionDTO, ProviderType::ProviderType},
};
use serde_json::{Value, json};
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	new_name:String,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Rename).await?;

	match provider {
		Some(registration) => {
			let response = super::InvokeProvider::Fn(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
					json!(new_name),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}
