//! `OutputChannelState::AddOrUpdate`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO, dev_log};

pub fn Fn(This:&Struct, id:String, channel:OutputChannelStateDTO) {
		if let Ok(mut guard) = This.OutputChannels.lock() {
			guard.insert(id, channel);

			dev_log!("output", "[OutputChannelState] Output channel added/updated");
		}
	}
