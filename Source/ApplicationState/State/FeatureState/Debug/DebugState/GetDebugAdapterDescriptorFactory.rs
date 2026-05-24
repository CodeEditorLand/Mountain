//! `DebugState::GetDebugAdapterDescriptorFactory`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(
		&self,

		DebugType:&str,
	) -> Option<DebugAdapterDescriptorFactoryRegistration> {
		This.DebugAdapterDescriptorFactories
			.lock()
			.ok()
			.and_then(|guard| guard.get(DebugType).cloned())
	}
