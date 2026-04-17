//! # LanguageFeature - Code Actions
//!
//! Provides code actions (quick fixes and refactorings) for a code range

#[allow(unused_imports)]
use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use serde_json::Value;
use tauri::{AppHandle, Wry};
use url::Url;

use super::{InvokeProvider::invoke_provider, validation::validate_language_feature_request};
use crate::dev_log;

/// Implementation of code actions command - called by the command wrapper in
/// the parent module.
pub(super) async fn provide_code_actions_impl(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
	context:Value,
) -> Result<Value, String> {
	dev_log!("commands", "[Language Feature] Providing code actions for: {} at {:?}", uri, position);

	validate_language_feature_request("code_actions", &uri, &position)?;

	let document_uri = Url::parse(&uri).map_err(|error| error.to_string())?;

	// Position is passed as RangeOrSelectionDTO (raw Value) per trait signature
	invoke_provider(application_handle, |provider| {
		async move {
			let result = provider
				.ProvideCodeActions(document_uri, position.clone(), context.clone())
				.await?;
			Ok(serde_json::to_value(result)?)
		}
	})
	.await
}
