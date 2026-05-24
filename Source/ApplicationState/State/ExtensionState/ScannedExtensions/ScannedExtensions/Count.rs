//! `ScannedExtensions::Count`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

pub fn Fn(This:&Struct) -> usize { This.ScannedExtensions.lock().ok().map(|guard| guard.len()).unwrap_or(0) }
