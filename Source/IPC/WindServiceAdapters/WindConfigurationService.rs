//! Wind-shaped configuration service - read / write of
//! configuration values via the injected
//! `ConfigurationProvider` trait. Defaults to user-target
//! writes; `ConfigurationOverridesDTO::default()` resolves to
//! the active scope.

use std::sync::Arc;

use CommonLibrary::Configuration::{
	ConfigurationProvider::ConfigurationProvider,
	DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
};

pub struct Struct {
	pub(super) provider:Arc<dyn ConfigurationProvider>,
}

impl Struct {
	pub fn new(provider:Arc<dyn ConfigurationProvider>) -> Self { Self { provider } }

	pub async fn get_value(&self, key:String) -> Result<serde_json::Value, String> {
		self.provider
			.GetConfigurationValue(Some(key.to_string()), ConfigurationOverridesDTO::default())
			.await
			.map_err(|e| e.to_string())
	}

	pub async fn update_value(&self, key:String, value:serde_json::Value) -> Result<(), String> {
		self.provider
			.UpdateConfigurationValue(
				key,
				value,
				ConfigurationTarget::User,
				ConfigurationOverridesDTO::default(),
				None,
			)
			.await
			.map_err(|e| e.to_string())
	}
}
