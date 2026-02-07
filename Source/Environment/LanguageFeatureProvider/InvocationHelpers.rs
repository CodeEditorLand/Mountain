//! Provider invocation and generic LSP feature method helper.

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::WriteFileBytes::WriteFileBytes,
	IPC::IPCProvider::IPCProvider,
};
use log::debug;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use url::Url;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Environment::Utility,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Finds the best provider for a given feature and document.
pub(super) async fn get_matching_provider(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: &Url,
	feature_type: ProviderType,
) -> Result<Option<ProviderRegistrationDTO>, CommonError> {
	let providers = environment
		.ApplicationState
		.Extension.ProviderRegistration.LanguageProviders
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	let document = environment
		.ApplicationState
		.Feature.Documents
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.get(document_uri.as_str())
		.cloned();

	if let Some(doc) = document {
		// Simplified selector matching - match on language identifier
		for provider in providers.values() {
			if provider.ProviderType == feature_type {
				if let Some(selector_array) = provider.Selector.as_array() {
					for selector in selector_array {
						if let Some(lang) = selector.get("language").and_then(|l| l.as_str()) {
							if lang == doc.LanguageIdentifier {
								debug!("Found provider with handle {} for document {}", provider.Handle, document_uri);
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

/// A generic helper to find the best provider, invoke it via RPC, and deserialize the result.
pub(super) async fn invoke_provider<TResponse: serde::de::DeserializeOwned>(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	provider_type: ProviderType,
	document_uri: &Url,
	mut provider_arguments: Value,
) -> Result<Option<TResponse>, CommonError> {
	if let Some(provider) = get_matching_provider(environment, document_uri, provider_type).await? {
		let rpc_method = format!("$provide{}", provider.ProviderType.to_string());

		let uri_components = json!({ "external": document_uri.to_string(), "$mid": 1 });

		let arguments_vector = provider_arguments.as_array_mut().ok_or_else(|| {
			CommonError::InvalidArgument {
				argument_name: "ProviderArguments".into(),
				reason: "Expected provider arguments to be a JSON array.".into(),
			}
		})?;

		let mut final_arguments_vector = vec![json!(provider.Handle), uri_components];
		final_arguments_vector.append(arguments_vector);

		let final_arguments = json!(final_arguments_vector);

		let ipc_provider: Arc<dyn IPCProvider> = environment.Require();

		let response = ipc_provider
			.SendRequestToSideCar(provider.SideCarIdentifier, rpc_method, final_arguments, 5000)
			.await?;

		if response.is_null() {
			return Ok(None);
		}
		serde_json::from_value(response).map_err(|error| {
			CommonError::SerializationError {
				description: format!("Failed to deserialize response for {:?}: {}", provider_type, error),
			}
		})
	} else {
		Ok(None)
	}
}
