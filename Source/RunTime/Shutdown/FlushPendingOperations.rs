//! Drain pending UI requests, replying with a "shutting down" error to each
//! awaiting caller so they unblock cleanly.

use CommonLibrary::Error::CommonError::CommonError;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

impl ApplicationRunTime {
	/// Flushs pending operations.
	pub async fn FlushPendingOperations(&self) {
		dev_log!("lifecycle", "[ApplicationRunTime] Flushing pending operations...");

		let mut PendingRequestsGuard = self.Environment.ApplicationState.UI.PendingUserInterfaceRequest.lock();

		for (_RequestIdentifier, Sender) in PendingRequestsGuard.drain() {
			let _ = Sender.send(Err(CommonError::Unknown {
				Description:"Application shutting down".to_string(),
			}));
		}

		dev_log!("lifecycle", "[ApplicationRunTime] Pending operations flushed");
	}
}
