//! Provides a file decoration (badge, tooltip, colour) for the given URI via
//! `$provideFileDecoration`. Called by Mountain's `FileDecorationProvider`
//! when the file explorer or source-control tree requests decoration state
//! for a resource URI.

use CommonLibrary::{Error::CommonError::CommonError, LanguageFeature::DTO::ProviderType::ProviderType};
use serde_json::{Value, json};
use url::Url;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	resource_uri:Url,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::super::ProviderLookup::get_matching_provider(environment, &resource_uri, ProviderType::FileDecoration)
			.await?;

	match provider {
		Some(registration) => {
			let response = super::InvokeProviderMethod::Fn(
				environment,
				&registration,
				"$provideFileDecoration",
				vec![
					json!(registration.Handle),
					json!({ "external": resource_uri.to_string(), "$mid": 1 }),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}
