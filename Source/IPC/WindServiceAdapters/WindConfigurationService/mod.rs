pub mod New;
pub mod GetValue;
pub mod UpdateValue;

use std::sync::Arc;
use CommonLibrary::Configuration::{
	ConfigurationProvider::ConfigurationProvider,
	DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
};

pub struct Struct {
	pub(super) provider:Arc<dyn ConfigurationProvider>,
}
