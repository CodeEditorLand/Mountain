//! `WebviewState::Count`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

pub fn Fn(This:&Struct) -> usize { This.ActiveWebviews.lock().ok().map(|guard| guard.len()).unwrap_or(0) }
