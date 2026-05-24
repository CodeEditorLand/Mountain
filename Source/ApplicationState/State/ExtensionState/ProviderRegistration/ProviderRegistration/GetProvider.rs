//! `ProviderRegistration::GetProvider`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

pub fn Fn(This:&Struct, handle:u32) -> Option<ProviderRegistrationDTO> {
		This.LanguageProviders.lock().ok().and_then(|guard| guard.get(&handle).cloned())
	}
