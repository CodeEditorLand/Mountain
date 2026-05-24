//! `DebugState::GetDebugSession`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct, SessionId:&str) -> Option<DebugSessionEntry> {
		This.DebugSessions.lock().ok().and_then(|Guard| Guard.get(SessionId).cloned())
	}
