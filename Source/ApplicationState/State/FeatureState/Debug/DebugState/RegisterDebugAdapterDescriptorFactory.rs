//! `DebugState::RegisterDebugAdapterDescriptorFactory`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(
		&self,

		DebugType:String,

		factory_handle:u32,

		sidecar_identifier:String,
	) -> Result<(), String> {
		let mut guard = self
			.DebugAdapterDescriptorFactories
			.lock()
			.map_err(|E| format!("Failed to lock debug adapter descriptor factories: {}", e))?;

		guard.insert(
			DebugType,
			DebugAdapterDescriptorFactoryRegistration {
				FactoryHandle:factory_handle,
				SideCarIdentifier:sidecar_identifier,
			},
		);

		Ok(())
	}
