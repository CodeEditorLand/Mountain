pub mod GetProviders;
pub mod GetProvider;
pub mod RegisterProvider;
pub mod UnregisterProvider;
pub mod GetProvidersForLanguage;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

/// Language provider registration state.
#[derive(Clone)]
pub struct Struct {
	/// Registered language providers by handle.
	pub LanguageProviders:Arc<StandardMutex<HashMap<u32, ProviderRegistrationDTO>>>,
}
