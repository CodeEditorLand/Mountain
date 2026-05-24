//! `DebugState::GetAllDebugAdapterDescriptorFactories`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct) -> HashMap<String, DebugAdapterDescriptorFactoryRegistration> {
		This.DebugAdapterDescriptorFactories
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}
