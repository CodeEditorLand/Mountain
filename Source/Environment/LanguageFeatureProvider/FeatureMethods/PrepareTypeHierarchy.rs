//! Prepares a type hierarchy - establishes the root `TypeHierarchyItem` at
//! the given document position via `$prepareTypeHierarchyItems`.

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
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::TypeHierarchy)
			.await?;

	match provider {
		Some(registration) => {
			let uri_json = json!({ "external": document_uri.to_string(), "$mid": 1 });

			let pos_json = json!({ "Line": position_dto.LineNumber, "Character": position_dto.Column });

			let response = super::InvokeProviderMethod::Fn(
				environment,
				&registration,
				"$prepareTypeHierarchyItems",
				vec![json!(registration.Handle), uri_json, pos_json],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}
