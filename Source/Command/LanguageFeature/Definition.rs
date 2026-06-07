//! # LanguageFeature - Definition
//!
//! Provides go-to-definition functionality

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

/// Implementation of definition command - called by the command wrapper in the
/// parent module.
pub(super) async fn provide_definition_impl(
	application_handle:AppHandle<Wry>,

	uri:String,

	position:Value,
) -> Result<Value, String> {

	dev_log!(
		"commands",

		"[Language Feature] Providing definition for: {} at {:?}",

		uri,

		position
	);

	validate_language_feature_request("definition", &uri, &position)?;

	let document_uri = Url::parse(&uri).map_err(|error| error.to_string())?;

	let position_dto:PositionDTO =
		serde_json::from_value(position.clone()).map_err(|error| format!("Failed to parse position: {}", error))?;

	invoke_provider(application_handle, |provider| {
		async move {
			let result = provider.ProvideDefinition(document_uri, position_dto).await?;

			Ok(serde_json::to_value(result)?)
		}
	})
	.await
}
