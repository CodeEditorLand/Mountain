//! `UIState::GetPendingRequests`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use CommonLibrary::Error::CommonError::CommonError;
use crate::dev_log;

pub fn Fn(This:&Struct) -> Vec<String> {
		This.PendingUserInterfaceRequest
			.lock()
			.ok()
			.map(|guard| guard.keys().cloned().collect())
			.unwrap_or_default()
	}
