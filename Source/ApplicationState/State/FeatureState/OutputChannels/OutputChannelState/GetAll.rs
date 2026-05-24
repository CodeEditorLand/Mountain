//! `OutputChannelState::GetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<String, OutputChannelStateDTO> {
		This.OutputChannels.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}
