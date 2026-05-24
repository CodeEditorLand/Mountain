//! `FlushPendingOperations::FlushPendingOperations`

use CommonLibrary::Error::CommonError::CommonError;

use super::Struct;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub fn Fn(This:&Struct) {
	dev_log!("lifecycle", "[ApplicationRunTime] Flushing pending operations...");

	let mut PendingRequestsGuard = self
		.Environment
		.ApplicationState
		.UI
		.PendingUserInterfaceRequest
		.lock()
		.unwrap_or_else(|E| {
			dev_log!(
				"lifecycle",
				"error: [ApplicationRunTime] Failed to lock pending UI requests: {}",
				E
			);
			E.into_inner()
		});

	for (_RequestIdentifier, Sender) in PendingRequestsGuard.drain() {
		let _ = Sender.send(Err(CommonError::Unknown {
			Description:"Application shutting down".to_string(),
		}));
	}

	dev_log!("lifecycle", "[ApplicationRunTime] Pending operations flushed");
}
