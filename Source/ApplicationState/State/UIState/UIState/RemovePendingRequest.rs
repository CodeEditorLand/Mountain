//! `UIState::RemovePendingRequest`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use CommonLibrary::Error::CommonError::CommonError;
use crate::dev_log;

pub fn Fn(
		&self,

		id:&str,
	) -> Option<tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>> {
		if let Ok(mut guard) = This.PendingUserInterfaceRequest.lock() {
			let sender = guard.remove(id);

			dev_log!("window", "[UIState] Pending UI request removed: {}", id);

			sender
		} else {
			None
		}
	}
