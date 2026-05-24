//! `DebugState::UnregisterDebugSession`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct, SessionId:&str) -> Option<DebugSessionEntry> {
		This.DebugSessions.lock().ok().and_then(|mut Guard| Guard.remove(SessionId))
	}
