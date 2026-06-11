//! Provides type-hierarchy subtypes for an item by invoking the matching
//! `TypeHierarchy` provider, resolved from the item's URI.

use CommonLibrary::{Error::CommonError::CommonError, LanguageFeature::DTO::ProviderType::ProviderType};
use serde_json::{Value, json};
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	let uri_str = item_dto.get("uri").and_then(|u| u.as_str()).unwrap_or("");

	let document_uri = Url::parse(uri_str).unwrap_or_else(|_| Url::parse("file:///unknown").unwrap());

	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::TypeHierarchy)
			.await?;

	match provider {
		Some(registration) => {
			let response =
				super::InvokeProvider::Fn(environment, &registration, vec![json!(registration.Handle), item_dto])
					.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}
