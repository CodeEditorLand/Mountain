
//! Wrap a Mountain command execution in an `INFO`-level span. Errors are
//! returned as `CommonError` (the project-wide error type from
//! `CommonLibrary`).

use CommonLibrary::Error::CommonError::CommonError;

#[cfg(feature = "Telemetry")]
use crate::dev_log;

#[cfg(feature = "Telemetry")]
pub async fn Fn<F, T>(CommandName:&str, Operation:F) -> Result<T, CommonError>
where
	F: std::future::Future<Output = Result<T, CommonError>>, {
	let Span = tracing::span!(
		tracing::Level::INFO,

		"command_execute",

		command = %CommandName
	);

	let _Enter = Span.enter();

	dev_log!("lifecycle", "Executing command: {}", CommandName);

	let Start = std::time::Instant::now();

	match Operation.await {
		Ok(Result) => {
			dev_log!(
				"lifecycle",
				"Command executed successfully: {} (duration: {:?})",
				CommandName,
				Start.elapsed()
			);

			Ok(Result)
		},

		Err(Err) => {
			dev_log!(
				"lifecycle",
				"error: Command execution failed: {} (duration: {:?}, error: {})",
				CommandName,
				Start.elapsed(),
				Err
			);

			Err(Err)
		},
	}
}

#[cfg(not(feature = "Telemetry"))]
pub async fn Fn<F, T>(_CommandName:&str, Operation:F) -> Result<T, CommonError>
where
	F: std::future::Future<Output = Result<T, CommonError>>, {
	Operation.await
}
