//! Provides workspace symbols for a query string. Workspace-symbol providers
//! are registered globally (no document URI), so the first registered
//! `WorkspaceSymbol` provider is invoked.

use CommonLibrary::{Error::CommonError::CommonError, LanguageFeature::DTO::ProviderType::ProviderType};
use serde_json::{Value, json};

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	query:String,
) -> Result<Option<Value>, CommonError> {
	// Workspace symbols don't have a specific document URI - use a dummy lookup.
	// The provider is registered globally, so we pick the first WorkspaceSymbol
	// provider.
	let MatchingRegistration = {
		let providers = environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.LanguageProviders
			.lock();

		providers
			.values()
			.find(|p| p.ProviderType == ProviderType::WorkspaceSymbol)
			.cloned()
	};

	match MatchingRegistration {
		Some(registration) => {
			let response =
				super::InvokeProvider::Fn(environment, &registration, vec![json!(registration.Handle), json!(query)])
					.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}
