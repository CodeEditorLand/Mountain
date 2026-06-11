//! Provides full-document semantic tokens by invoking the matching
//! `SemanticTokens` provider in its owning sidecar.

use CommonLibrary::{Error::CommonError::CommonError, LanguageFeature::DTO::ProviderType::ProviderType};
use serde_json::{Value, json};
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::SemanticTokens)
			.await?;

	match provider {
		Some(registration) => {
			let response = super::InvokeProvider::Fn(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}
