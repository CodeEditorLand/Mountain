//! `ProviderRegistration::GetProviders`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<u32, ProviderRegistrationDTO> {
		This.LanguageProviders
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}
