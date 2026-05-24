//! `DecorationsState::GetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&Struct) -> HashMap<String, Value> {
		This.Entries.lock().ok().map(|Guard| Guard.clone()).unwrap_or_default()
	}
