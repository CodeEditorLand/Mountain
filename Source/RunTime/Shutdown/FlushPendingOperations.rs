//! Drain pending UI requests, replying with a "shutting down" error to each
//! awaiting caller so they unblock cleanly.

use CommonLibrary::Error::CommonError::CommonError;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

impl ApplicationRunTime {
	pub async fn FlushPendingOperations(&self) {
		dev_log!("lifecycle", "[ApplicationRunTime] Flushing pending operations...");

<<<<<<< HEAD
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
=======
		let mut PendingRequestsGuard = self.Environment.ApplicationState.UI.PendingUserInterfaceRequest.lock();
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

		for (_RequestIdentifier, Sender) in PendingRequestsGuard.drain() {
			let _ = Sender.send(Err(CommonError::Unknown {
				Description:"Application shutting down".to_string(),
			}));
		}

		dev_log!("lifecycle", "[ApplicationRunTime] Pending operations flushed");
	}
}
