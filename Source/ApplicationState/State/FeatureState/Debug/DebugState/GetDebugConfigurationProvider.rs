//! `DebugState::GetDebugConfigurationProvider`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct, DebugType:&str) -> Option<DebugConfigurationProviderRegistration> {
		This.DebugConfigurationProviders
			.lock()
			.ok()
			.and_then(|guard| guard.get(DebugType).cloned())
	}
