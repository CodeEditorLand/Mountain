//! `ProviderRegistration::GetProvidersForLanguage`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

pub fn Fn(This:&Struct, language:&str) -> Vec<ProviderRegistrationDTO> {
		This.LanguageProviders
			.lock()
			.ok()
			.map(|guard| guard.values().filter(|p| p.MatchesSelector("", language)).cloned().collect())
			.unwrap_or_default()
	}
