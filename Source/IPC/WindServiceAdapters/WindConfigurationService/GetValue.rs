//! `WindConfigurationService::GetValue`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::Configuration::{
	ConfigurationProvider::ConfigurationProvider,
	DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
};

pub fn Fn(This:&Struct, key:String) -> Result<serde_json::Value, String> {
		This.provider
			.GetConfigurationValue(Some(key.to_string()), ConfigurationOverridesDTO::default())
			.await
			.map_err(|E| e.to_string())
	}
