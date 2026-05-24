//! `RouteMessage::RouteMessage`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};
use crate::dev_log;

pub fn Fn(This:&Struct, Message:&TauriIPCMessage) -> Result<(), String> {

		dev_log!("ipc", "[Router] Routing message on channel: {}", Message.channel);

		// Validate message before routing
		Message.Validate().map_err(|E| format!("Message validation failed: {}", e))?;

		let listeners = self
			.listeners
			.lock()
			.map_err(|E| format!("Failed to access listeners: {}", e))?;

		let listeners_map = &*listeners;

		let channel_listeners = listeners_map.get(&Message.channel);

		if let Some(channel_listeners) = channel_listeners {

			let listener_count = channel_listeners.len();

			let mut success_count = 0;

			let mut error_count = 0;

			for (index, callback) in channel_listeners.iter().enumerate() {

				let message_data = Message.data.clone();

				match callback(message_data) {

					Ok(_) => success_count += 1,

					Err(e) => {

						dev_log!(
							"ipc",

							"error: [Router] Error in listener {} for channel {}: {}",

							index,

							Message.channel,

							e
						);

						error_count += 1;
					},
				}
			}

			dev_log!(
				"ipc",

				"[Router] Message routed to channel {}: {}/{} listeners succeeded",

				Message.channel,

				success_count,

				listener_count
			);

			if error_count > 0 {

				dev_log!(
					"ipc",

					"warn: [Router] {} listener(s) failed on channel {}",

					error_count,

					Message.channel
				);
			}
		} else {

			dev_log!("ipc", "[Router] No listeners found for channel: {}", Message.channel);
		}

		Ok(())
	}
