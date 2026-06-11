//! Provides selection ranges for a set of positions by invoking the matching
//! `SelectionRange` provider in its owning sidecar.

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::DTO::{PositionDTO::PositionDTO, ProviderType::ProviderType},
};
use serde_json::{Value, json};
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	positions:Vec<PositionDTO>,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::SelectionRange)
			.await?;

	match provider {
		Some(registration) => {
			let response = super::InvokeProvider::Fn(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(positions),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}
