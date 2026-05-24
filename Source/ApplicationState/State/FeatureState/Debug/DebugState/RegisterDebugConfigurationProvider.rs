//! `DebugState::RegisterDebugConfigurationProvider`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(
		&self,

		DebugType:String,

		ProviderHandle:u32,

		sidecar_identifier:String,
	) -> Result<(), String> {
		let mut guard = self
			.DebugConfigurationProviders
			.lock()
			.map_err(|E| format!("Failed to lock debug configuration providers: {}", e))?;

		guard.insert(
			DebugType,
			DebugConfigurationProviderRegistration { ProviderHandle, SideCarIdentifier:sidecar_identifier },
		);

		Ok(())
	}
