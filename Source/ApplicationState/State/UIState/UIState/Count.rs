//! `UIState::Count`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use CommonLibrary::Error::CommonError::CommonError;
use crate::dev_log;

pub fn Fn(This:&Struct) -> usize {
		This.PendingUserInterfaceRequest
			.lock()
			.ok()
			.map(|guard| guard.len())
			.unwrap_or(0)
	}
