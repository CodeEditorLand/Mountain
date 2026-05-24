//! `DebugState::RegisterDebugSession`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct, Entry:DebugSessionEntry) -> Result<(), String> {
		let mut Guard = self
			.DebugSessions
			.lock()
			.map_err(|Error| format!("Failed to lock DebugSessions: {}", Error))?;

		Guard.insert(Entry.SessionId.clone(), Entry);

		Ok(())
	}
