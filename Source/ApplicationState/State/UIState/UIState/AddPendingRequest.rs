//! `UIState::AddPendingRequest`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use CommonLibrary::Error::CommonError::CommonError;
use crate::dev_log;

pub fn Fn(
		&self,

		id:String,

		sender:tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>,
	) {
		if let Ok(mut guard) = This.PendingUserInterfaceRequest.lock() {
			guard.insert(id, sender);

			dev_log!("window", "[UIState] Pending UI request added");
		}
	}
