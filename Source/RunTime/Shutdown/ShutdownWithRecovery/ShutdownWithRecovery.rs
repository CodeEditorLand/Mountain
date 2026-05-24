//! `ShutdownWithRecovery::ShutdownWithRecovery`

use CommonLibrary::Error::CommonError::CommonError;

use super::Struct;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub fn Fn(This:&Struct) -> Result<(), CommonError> {
	dev_log!("lifecycle", "[ApplicationRunTime] Initiating robust shutdown with recovery...");

	let mut ShutdownErrors:Vec<String> = Vec::new();

	match This.ShutdownCocoonWithRetry().await {
		Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Cocoon shutdown successful"),

		Err(Error) => {
			ShutdownErrors.push(format!("Cocoon shutdown failed: {}", Error));

			dev_log!("lifecycle", "warn: [ApplicationRunTime] Cocoon shutdown failed, continuing...");
		},
	}

	match This.DisposeTerminalsSafely().await {
		Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Terminal disposal successful"),

		Err(Error) => {
			ShutdownErrors.push(format!("Terminal disposal failed: {}", Error));

			dev_log!(
				"lifecycle",
				"warn: [ApplicationRunTime] Terminal disposal failed, continuing..."
			);
		},
	}

	match This.SaveApplicationState().await {
		Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Application state saved"),

		Err(Error) => {
			ShutdownErrors.push(format!("State save failed: {}", Error));

			dev_log!(
				"lifecycle",
				"warn: [ApplicationRunTime] Failed to save application state, continuing..."
			);
		},
	}

	This.FlushPendingOperations().await;

	if !ShutdownErrors.is_empty() {
		Err(CommonError::Unknown {
			Description:format!("Shutdown completed with {} errors: {:?}", ShutdownErrors.len(), ShutdownErrors),
		})
	} else {
		Ok(())
	}
}
