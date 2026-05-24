//! `ProviderRegistration::UnregisterProvider`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

pub fn Fn(This:&Struct, handle:u32) {
		if let Ok(mut guard) = This.LanguageProviders.lock() {
			guard.remove(&handle);

			dev_log!(
				"extensions",
				"[ProviderRegistration] Provider unregistered with handle: {}",
				handle
			);
		}
	}
