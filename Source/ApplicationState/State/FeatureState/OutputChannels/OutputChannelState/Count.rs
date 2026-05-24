//! `OutputChannelState::Count`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO, dev_log};

pub fn Fn(This:&Struct) -> usize { This.OutputChannels.lock().ok().map(|guard| guard.len()).unwrap_or(0) }
