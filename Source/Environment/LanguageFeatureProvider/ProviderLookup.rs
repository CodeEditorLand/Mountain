//! Provider lookup and matching utilities.

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::DTO::ProviderType::ProviderType,
};
use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;
use crate::Environment::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError;
use log::warn;
use url::Url;

pub(super) async fn get_matching_provider(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: &Url,
	feature_type: ProviderType,
) -> Result<Option<ProviderRegistrationDTO>, CommonError> {
	let providers = environment.ApplicationState.LanguageProviders.lock().map_err(MapApplicationStateLockErrorToCommonError)?;
	let open_documents = environment.ApplicationState.OpenDocuments.lock().map_err(MapApplicationStateLockErrorToCommonError)?;

	if let Some(document) = open_documents.get(document_uri.as_str()) {
		for provider in providers.values() {
			if provider.ProviderType == feature_type {
				if let Some(selector_array) = provider.Selector.as_array() {
					for selector in selector_array {
						if let Some(language) = selector.get("language").and_then(|l| l.as_str()) {
							if language == document.LanguageIdentifier {
								return Ok(Some(provider.clone()));
							}
						}
					}
				}
			}
		}
	}

	warn!("No provider found for {:?} on document {}", feature_type, document_uri);
	Ok(None)
}
