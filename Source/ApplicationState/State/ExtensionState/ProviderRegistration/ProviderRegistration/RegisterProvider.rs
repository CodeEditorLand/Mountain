//! `ProviderRegistration::RegisterProvider`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

pub fn Fn(This:&Struct, handle:u32, provider:ProviderRegistrationDTO) {
		if let Ok(mut guard) = This.LanguageProviders.lock() {
			guard.insert(handle, provider);

			// Duplicate of the `provider-register` log line emitted by
			// `MountainVinegRPCService`'s OR match - both fire per
			// provider registration. Route this one to
			// `provider-register` too (now short-muted) so the
			// `extensions` tag stays signal-only (scan start/end,
			// classification changes, Install events).
			dev_log!(
				"provider-register",
				"[ProviderRegistration] Provider registered with handle: {}",
				handle
			);
		}
	}
