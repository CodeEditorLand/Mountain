//! # LanguageFeature - References
//!
//! Finds all references to a symbol

#[allow(unused_imports)]
use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::{
		DTO::PositionDTO::PositionDTO,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};
use serde_json::Value;
use tauri::{AppHandle, Wry};
use url::Url;

use super::{InvokeProvider::invoke_provider, Validation::validate_language_feature_request};
use crate::dev_log;

/// Implementation of references command - called by the command wrapper in the
/// parent module.
pub(super) async fn provide_references_impl(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
	context:Value,
) -> Result<Value, String> {
	dev_log!("commands", "[Language Feature] Providing references for: {} at {:?}", uri, position);

	validate_language_feature_request("references", &uri, &position)?;

	let document_uri = Url::parse(&uri).map_err(|error| error.to_string())?;

	let position_dto:PositionDTO =
		serde_json::from_value(position.clone()).map_err(|error| format!("Failed to parse position: {}", error))?;

	// Context is passed as raw Value per trait signature
	invoke_provider(application_handle, |provider| {
		async move {
			let result = provider.ProvideReferences(document_uri, position_dto, context.clone()).await?;
			Ok(serde_json::to_value(result)?)
		}
	})
	.await
}
