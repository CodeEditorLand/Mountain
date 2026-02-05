//! # LanguageFeature - Hover
//!
//! Provides hover information at cursor position

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::{
		DTO::PositionDTO::PositionDTO,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};
use log::debug;
use serde_json::Value;
use tauri::{AppHandle, Wry};
use url::Url;

use super::{invoke_provider::invoke_provider, validation::validate_language_feature_request};

/// Implementation of hover command - called by the command wrapper in the parent module.
pub(super) async fn provide_hover_impl(
	application_handle: AppHandle<Wry>,
	uri: String,
	position: Value,
) -> Result<Value, String> {
	debug!("[Language Feature] Providing hover for: {} at {:?}", uri, position);

	validate_language_feature_request("hover", &uri, &position)?;

	let document_uri = Url::parse(&uri).map_err(|error| error.to_string())?;

	let position_dto: PositionDTO = serde_json::from_value(position.clone())
		.map_err(|error| format!("Failed to parse position: {}", error))?;

	invoke_provider(application_handle, |provider| {
		async move {
			let result = provider.ProvideHover(document_uri, position_dto).await?;
			Ok(serde_json::to_value(result)?)
		}
	})
	.await
}
