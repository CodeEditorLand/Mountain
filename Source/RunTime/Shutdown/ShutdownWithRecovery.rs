//! Robust shutdown that continues across individual service failures.
//! Cocoon retry → terminal disposal → state save → flush. Errors collected
//! into one summary instead of crashing.

use CommonLibrary::Error::CommonError::CommonError;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

impl ApplicationRunTime {
	pub async fn ShutdownWithRecovery(&self) -> Result<(), CommonError> {
		dev_log!("lifecycle", "[ApplicationRunTime] Initiating robust shutdown with recovery...");

		let mut ShutdownErrors:Vec<String> = Vec::new();

		match self.ShutdownCocoonWithRetry().await {
			Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Cocoon shutdown successful"),

			Err(Error) => {
				ShutdownErrors.push(format!("Cocoon shutdown failed: {}", Error));

				dev_log!("lifecycle", "warn: [ApplicationRunTime] Cocoon shutdown failed, continuing...");
			},
		}

		match self.DisposeTerminalsSafely().await {
			Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Terminal disposal successful"),

			Err(Error) => {
				ShutdownErrors.push(format!("Terminal disposal failed: {}", Error));

				dev_log!(
					"lifecycle",
					"warn: [ApplicationRunTime] Terminal disposal failed, continuing..."
				);
			},
		}

		match self.SaveApplicationState().await {
			Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Application state saved"),

			Err(Error) => {
				ShutdownErrors.push(format!("State save failed: {}", Error));

				dev_log!(
					"lifecycle",
					"warn: [ApplicationRunTime] Failed to save application state, continuing..."
				);
			},
		}

		self.FlushPendingOperations().await;

		if !ShutdownErrors.is_empty() {
			Err(CommonError::Unknown {
				Description:format!("Shutdown completed with {} errors: {:?}", ShutdownErrors.len(), ShutdownErrors),
			})
		} else {
			Ok(())
		}
	}
}
