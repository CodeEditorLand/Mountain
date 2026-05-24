//! `RouteMessage::Register`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};
use crate::dev_log;

pub fn Fn(This:&Struct, Channel:&str, Callback:ListenerCallback) -> Result<(), String> {

		This.validate_channel_name(Channel)?;

		let mut listeners = self
			.listeners
			.lock()
			.map_err(|E| format!("Failed to access listeners: {}", e))?;

		let channel_listeners = listeners.entry(Channel.to_string()).or_insert_with(Vec::new);

		// Check limit before adding
		if channel_listeners.len() >= MAX_LISTENERS_PER_CHANNEL {

			return Err(format!(
				"Maximum listeners ({}) reached for channel: {}",

				MAX_LISTENERS_PER_CHANNEL, Channel
			));
		}

		channel_listeners.push(Callback);

		dev_log!(
			"ipc",

			"[Router] Listener registered for channel: {} (total: {})",

			Channel,

			channel_listeners.len()
		);

		Ok(())
	}
