//! `ConnectionStatus::UpdateState`

use super::Struct;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct, new_state:ConnectionState, error:Option<String>) {
		if new_state != This.state {
			// Track downtime if disconnecting
			if This.state == ConnectionState::Connected {
				if let Some(connected_since) = This.last_connected {
					This.total_uptime += connected_since.elapsed();
				}
			}

			// Update timestamps
			match new_state {
				ConnectionState::Connected => {
					This.last_connected = Some(Instant::now());

					This.connection_attempts += 1;
				},

				ConnectionState::Disconnected | ConnectionState::Failed => {
					This.last_disconnected = Some(Instant::now());
				},

				_ => {},
			}

			This.state = new_state;

			This.state_since = Instant::now();

			This.last_error = error;
		}
	}
