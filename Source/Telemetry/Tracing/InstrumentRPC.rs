
//! Wrap an async gRPC call in an `INFO`-level span and emit start/finish
//! `dev_log` lines with elapsed time. Pass-through when `Telemetry` is
//! off.

#[cfg(feature = "Telemetry")]
use crate::dev_log;

#[cfg(feature = "Telemetry")]
pub async fn Fn<F, T, E>(ServiceName:&str, MethodName:&str, Operation:F) -> Result<T, E>
where
	F: std::future::Future<Output = Result<T, E>>,
	E: std::fmt::Display, {
	let Span = tracing::span!(
		tracing::Level::INFO,

		"rpc_call",

		service = %ServiceName,

		method = %MethodName
	);

	let _Enter = Span.enter();

	dev_log!("lifecycle", "RPC call started: {}.{}", ServiceName, MethodName);

	let Start = std::time::Instant::now();

	match Operation.await {
		Ok(Result) => {
			dev_log!(
				"lifecycle",
				"RPC call completed: {}.{} (duration: {:?})",
				ServiceName,
				MethodName,
				Start.elapsed()
			);

			Ok(Result)
		},

		Err(Err) => {
			dev_log!(
				"lifecycle",
				"error: RPC call failed: {}.{} (duration: {:?}, error: {})",
				ServiceName,
				MethodName,
				Start.elapsed(),
				Err
			);

			Err(Err)
		},
	}
}

#[cfg(not(feature = "Telemetry"))]
pub async fn Fn<F, T, E>(_ServiceName:&str, _MethodName:&str, Operation:F) -> Result<T, E>
where
	F: std::future::Future<Output = Result<T, E>>, {
	Operation.await
}
