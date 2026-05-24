//! `RouteMessage::ClearAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};
use crate::dev_log;

pub fn Fn(This:&Struct) -> Result<(), String> {

		let mut listeners = self
			.listeners
			.lock()
			.map_err(|E| format!("Failed to access listeners: {}", e))?;

		let total_listeners:usize = listeners.values().map(|l| l.len()).sum();

		listeners.clear();

		dev_log!(
			"ipc",

			"[Router] Cleared {} listeners from {} channels",

			total_listeners,

			listeners.len()
		);

		Ok(())
	}
