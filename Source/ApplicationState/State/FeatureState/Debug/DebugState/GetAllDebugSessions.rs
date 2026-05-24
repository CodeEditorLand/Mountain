//! `DebugState::GetAllDebugSessions`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct) -> HashMap<String, DebugSessionEntry> {
		This.DebugSessions.lock().ok().map(|Guard| Guard.clone()).unwrap_or_default()
	}
