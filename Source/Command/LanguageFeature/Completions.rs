//! # LanguageFeature - Completions
//!
//! Provides code completion suggestions

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::{
		DTO::{CompletionContextDTO::CompletionContextDTO, PositionDTO::PositionDTO},
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};
use serde_json::Value;
use tauri::{AppHandle, Wry};
use url::Url;

use super::{InvokeProvider::invoke_provider, Validation::validate_language_feature_request};
use crate::dev_log;

/// Implementation of completions command - called by the command wrapper in the
/// parent module.
pub(super) async fn provide_completions_impl(
	application_handle:AppHandle<Wry>,

	uri:String,

	position:Value,

	context:Value,
) -> Result<Value, String> {
	dev_log!(
		"commands",
		"[Language Feature] Providing completions for: {} at {:?}",
		uri,
		position
	);

	validate_language_feature_request("completions", &uri, &position)?;

	let document_uri = Url::parse(&uri).map_err(|error| error.to_string())?;

	let position_dto:PositionDTO =
		serde_json::from_value(position.clone()).map_err(|error| format!("Failed to parse position: {}", error))?;

	let context_dto:CompletionContextDTO =
		serde_json::from_value(context.clone()).map_err(|error| format!("Failed to parse context: {}", error))?;

	invoke_provider(application_handle, |provider| {
		async move {
			// Cancellation token currently not used, pass None
			let result = provider
				.ProvideCompletions(document_uri, position_dto, context_dto, None)
				.await?;

			Ok(serde_json::to_value(result)?)
		}
	})
	.await
}
