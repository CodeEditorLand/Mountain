//! Retry a recovery `Operation` up to `MaxAttempts` times with
//! exponential backoff (100 ms, doubled per failure). The async
//! sleep yields the runtime so other work can proceed during the
//! retry window. Final failure surfaces the last error verbatim.

use CommonLibrary::Error::CommonError::CommonError;

use crate::dev_log;

/// fn.
pub async fn Fn<F, T>(Operation:F, MaxAttempts:u32, OperationName:&str) -> Result<T, CommonError>
where
	F: Fn() -> Result<T, CommonError> + Send, {
	let mut Attempt = 0;

	let mut DelayMs:u64 = 100;

	while Attempt < MaxAttempts {
		match Operation() {
			Ok(Result) => return Ok(Result),

			Err(Error) => {
				Attempt += 1;

				if Attempt == MaxAttempts {
					return Err(Error);
				}

				dev_log!(
					"lifecycle",
					"warn: [RecoverState] Attempt {} failed for '{}': {}. Retrying in {}ms...",
					Attempt,
					OperationName,
					Error,
					DelayMs
				);

				tokio::time::sleep(tokio::time::Duration::from_millis(DelayMs)).await;

				DelayMs *= 2;
			},
		}
	}

	Err(CommonError::Unknown {
		Description:format!("Failed to recover state for '{}' after {} attempts", OperationName, MaxAttempts),
	})
}
