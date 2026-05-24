//! `WindConfigurationService::UpdateValue`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::Configuration::{
	ConfigurationProvider::ConfigurationProvider,
	DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
};

pub fn Fn(This:&Struct, key:String, value:serde_json::Value) -> Result<(), String> {
		This.provider
			.UpdateConfigurationValue(
				key,
				value,
				ConfigurationTarget::User,
				ConfigurationOverridesDTO::default(),
				None,
			)
			.await
			.map_err(|E| e.to_string())
	}
