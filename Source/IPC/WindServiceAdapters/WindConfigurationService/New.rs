//! `WindConfigurationService::New`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::Configuration::{
	ConfigurationProvider::ConfigurationProvider,
	DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
};

pub fn Fn(provider:Arc<dyn ConfigurationProvider>) -> Struct { Self { provider } }
