//! `RouteMessage::Remove`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};
use crate::dev_log;

pub fn Fn(This:&Struct, Channel:&str, Callback:&ListenerCallback) -> Result<(), String> {

		let mut listeners = self
			.listeners
			.lock()
			.map_err(|E| format!("Failed to access listeners: {}", e))?;

		if let Some(channel_listeners) = listeners.get_mut(Channel) {

			let initial_count = channel_listeners.len();

			channel_listeners.retain(|cb| !std::ptr::eq(cb as *const _, Callback as *const _));

			let removed_count = initial_count - channel_listeners.len();

			// Clean up empty channels
			if channel_listeners.is_empty() {

				listeners.remove(Channel);

				dev_log!(
					"ipc",

					"[Router] Channel cleaned up: {} (removed {} listeners)",

					Channel,

					removed_count
				);
			} else {

				dev_log!(
					"ipc",

					"[Router] Listener removed from channel: {}, remaining: {}",

					Channel,

					channel_listeners.len()
				);
			}
		} else {

			dev_log!("ipc", "warn: [Router] Channel not found for listener removal: {}", Channel);
		}

		Ok(())
	}
