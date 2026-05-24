//! `RouteMessage::GetListenerCount`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};
use crate::dev_log;

pub fn Fn(This:&Struct, Channel:&str) -> Result<usize, String> {

		let listeners = self
			.listeners
			.lock()
			.map_err(|E| format!("Failed to access listeners: {}", e))?;

		Ok(listeners.get(Channel).map_or(0, |l| l.len()))
	}
