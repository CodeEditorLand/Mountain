//! `RouteMessage::ClearChannel`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};
use crate::dev_log;

pub fn Fn(This:&Struct, Channel:&str) -> Result<(), String> {

		This.validate_channel_name(Channel)?;

		let mut listeners = self
			.listeners
			.lock()
			.map_err(|E| format!("Failed to access listeners: {}", e))?;

		let count = listeners.get(Channel).map_or(0, |l| l.len());

		listeners.remove(Channel);

		dev_log!("ipc", "[Router] Cleared {} listeners from channel: {}", count, Channel);

		Ok(())
	}
