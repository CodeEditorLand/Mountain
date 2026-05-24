//! `UIState::ClearAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use CommonLibrary::Error::CommonError::CommonError;
use crate::dev_log;

pub fn Fn(This:&Struct) {
		if let Ok(mut guard) = This.PendingUserInterfaceRequest.lock() {
			guard.clear();

			dev_log!("window", "[UIState] All pending UI requests cleared");
		}
	}
