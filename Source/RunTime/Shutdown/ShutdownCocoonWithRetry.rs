#![allow(non_snake_case)]

//! Send `$shutdown` over gRPC to Cocoon (3 attempts), then SIGKILL the child
//! regardless of gRPC outcome. The hard-kill (Atom I6) is critical: a gRPC
//! failure (transport error, broken pipe) used to leave the child orphaned,
//! holding port 50052, and the next Mountain launch hit EADDRINUSE with the
//! extension host stuck in degraded mode.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires, Error::CommonError::CommonError, IPC::IPCProvider::IPCProvider,
};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

impl ApplicationRunTime {
	pub async fn ShutdownCocoonWithRetry(&self) -> Result<(), CommonError> {
		let IPCProvider:Arc<dyn IPCProvider> = self.Environment.Require();

		let MaximumAttempts = 3;
		let mut Attempts = 0;
		let mut GracefulOk = false;
		let mut LastError:Option<CommonError> = None;

		while Attempts < MaximumAttempts {
			match IPCProvider
				.SendNotificationToSideCar(
					"cocoon-main".to_string(),
					"$shutdown".to_string(),
					serde_json::Value::Null,
				)
				.await
			{
				Ok(()) => {
					tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
					GracefulOk = true;
					break;
				},
				Err(Error) => {
					Attempts += 1;
					LastError = Some(Error.clone());
					if Attempts < MaximumAttempts {
						dev_log!(
							"lifecycle",
							"warn: [ApplicationRunTime] Cocoon shutdown attempt {} failed: {}. Retrying...",
							Attempts,
							Error
						);
						tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
					}
				},
			}
		}

		// Mark the Vine gRPC client shutting down BEFORE the SIGKILL so any
		// background tokio task firing `SendNotification` after this flips
		// short-circuits to `Ok(())` instead of attempting a TCP connect to
		// the dead socket and logging a false-positive `Connection refused`.
		crate::Vine::Client::MarkShutdown();

		// Atom I6: always reap the child after the graceful attempt. No-op if
		// the child already exited from $shutdown.
		crate::ProcessManagement::CocoonManagement::HardKillCocoon().await;

		if GracefulOk {
			Ok(())
		} else {
			Err(LastError.unwrap_or_else(|| {
				CommonError::Unknown {
					Description:"Failed to shutdown Cocoon after maximum retries".to_string(),
				}
			}))
		}
	}
}
