//! `OutputChannelState::Remove`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) {
		if let Ok(mut guard) = This.OutputChannels.lock() {
			guard.remove(id);

			dev_log!("output", "[OutputChannelState] Output channel removed: {}", id);
		}
	}
