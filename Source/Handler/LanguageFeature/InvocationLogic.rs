// @module InvocationLogic (language_feature/Handler)
// @description Contains the generic logic for finding and invoking a suitable
// language feature provider.

use Common::{error::CommonError, language_feature::DTO::ProviderType};
use log::{debug, warn};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::ProviderRegistrationDTO},
	Vine::client,
};

// Finds the best provider for a given feature and document.
//
// NOTE: This is a highly simplified stub. A real implementation needs a robust
// system to parse the document selector (glob patterns, language ID, scheme)
// and match it against the document's properties to score and select the best
// provider.
fn find_best_provider(
	app_state:&ApplicationState,
	provider_type:ProviderType,
	document_uri:&Url,
	language_id:&str,
) -> Option<ProviderRegistrationDTO> {
	let providers = app_state.LanguageProviders.lock().unwrap();
	// Extremely basic filtering logic for demonstration.
	for provider in providers.values() {
		if provider.ProviderType == provider_type {
			// This should check the document selector.
			// For now, we just return the first one we find.
			debug!("Found provider with handle {} for document {}", provider.Handle, document_uri);
			return Some(provider.clone());
		}
	}
	warn!(
		"No provider found for {:?} on document {} with lang id {}",
		provider_type, document_uri, language_id
	);
	None
}

// A generic helper to find the best provider for a feature, invoke it via RPC,
// and deserialize the result.
pub async fn InvokeProvider<R:Runtime, T:DeserializeOwned>(
	app_handle:&AppHandle<R>,
	provider_type:ProviderType,
	document_uri:&Url,
	language_id:&str,
	mut provider_args:Value,
) -> Result<Option<T>, CommonError> {
	let app_state = app_handle.state::<ApplicationState>();

	if let Some(provider) = find_best_provider(&app_state, provider_type, document_uri, language_id) {
		let rpc_method = format!("${}", provider_type.to_string()); // e.g., $provideHover
		let uri_components = json!({ "external": document_uri.to_string(), "$mid": 1 });

		// Prepend the provider handle and URI to the arguments list for the RPC call.
		let final_args = {
			let mut args_vec = provider_args.as_array_mut().ok_or_else(|| {
				CommonError::InvalidArg {
					ArgumentName:"provider_args".into(),
					Reason:"Expected provider arguments to be a JSON array.".into(),
				}
			})?;
			let mut final_vec = vec![json!(provider.Handle), uri_components];
			final_vec.append(args_vec);
			json!(final_vec)
		};

		debug!(
			"Invoking {} on sidecar '{}' for provider handle {}",
			rpc_method, provider.SidecarIdentifier, provider.Handle
		);

		let response = client::SendRequest(provider.SidecarIdentifier, rpc_method, final_args, 5000).await?; // 5-second timeout

		if response.is_null() {
			return Ok(None);
		}

		serde_json::from_value(response).map_err(|e| {
			CommonError::SerdeError {
				Description:format!("Failed to deserialize response for {:?}: {}", provider_type, e),
			}
		})
	} else {
		Ok(None)
	}
}
