//! `DecorationsState::GetDecoration`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&Struct, Uri:&str) -> Option<Value> {
		This.Entries.lock().ok().and_then(|Guard| Guard.get(Uri).cloned())
	}
