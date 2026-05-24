//! `OutputChannelState::Clear`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO, dev_log};

pub fn Fn(This:&Struct) {
		if let Ok(mut guard) = This.OutputChannels.lock() {
			guard.clear();

			dev_log!("output", "[OutputChannelState] All output channels cleared");
		}
	}
