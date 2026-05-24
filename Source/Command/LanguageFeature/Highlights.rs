//! # LanguageFeature - Document Highlights
//!
//! Finds symbol occurrences (document highlights) in a document

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

use super::{InvokeProvider::InvokeProvider, Validation::validate_language_feature_request};
use crate::dev_log;

/// Implementation of document highlights command - called by the command
/// wrapper in the parent module.
pub(super) async fn provide_document_highlights_impl(
	application_handle:AppHandle<Wry>,

	uri:String,

	position:Value,
) -> Result<Value, String> {
	dev_log!(
		"commands",
		"[Language Feature] Providing document highlights for: {} at {:?}",
		uri,
		position
	);

	validate_language_feature_request("document_highlights", &uri, &position)?;

	let document_uri = Url::parse(&uri).map_err(|Error| error.to_string())?;

	let position_dto:PositionDTO =
		serde_json::from_value(position.clone()).map_err(|Error| format!("Failed to parse position: {}", error))?;

	InvokeProvider(application_handle, |provider| {
		async move {
			let result = provider.ProvideDocumentHighlights(document_uri, position_dto).await?;
			Ok(serde_json::to_value(result)?)
		}
	})
	.await
}
