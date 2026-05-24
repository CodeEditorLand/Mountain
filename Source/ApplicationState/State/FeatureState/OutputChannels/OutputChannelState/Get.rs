//! `OutputChannelState::Get`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) -> Option<OutputChannelStateDTO> {
		This.OutputChannels.lock().ok().and_then(|guard| guard.get(id).cloned())
	}
