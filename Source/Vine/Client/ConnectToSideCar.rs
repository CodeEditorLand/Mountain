//! Establish a gRPC connection to a Cocoon sidecar with exponential
//! back-off retry. On success initialises the per-connection metadata
//! tracked by `Shared::CONNECTION_METADATA`.

use std::time::{Duration, Instant};

use crate::{
	Vine::{
		Client::{
			Shared::{
				CONNECTION_METADATA,
				ConnectionMetadata,
				FireConnectionNotify,
				MAX_RETRY_ATTEMPTS,
				RETRY_BASE_DELAY_MS,
			},
			TryConnectSingle,
		},
		Error::VineError,
	},
	dev_log,
};

pub async fn Fn(SideCarIdentifier:String, Address:String) -> Result<(), VineError> {
	dev_log!(
		"grpc",
		"[VineClient] Connecting to sidecar '{}' at '{}'...",
		SideCarIdentifier,
		Address
	);

	let Endpoint = format!("http://{}", Address);

	if Endpoint.len() > 256 {
		return Err(VineError::RPCError(
			"Invalid endpoint address: exceeds maximum length".to_string(),
		));
	}

	let mut LastError = None;

	for Attempt in 1..=MAX_RETRY_ATTEMPTS {
		let Result = TryConnectSingle::Fn(&SideCarIdentifier, &Endpoint).await;

		if Result.is_ok() {
			CONNECTION_METADATA.lock().insert(
				SideCarIdentifier.clone(),
				ConnectionMetadata { LastActivity:Instant::now(), FailureCount:0, IsHealthy:true },
			);

			dev_log!("grpc", "[VineClient] Successfully connected to sidecar '{}'", SideCarIdentifier);

			// Unblock any `WaitForClientConnection` callers immediately.
			FireConnectionNotify(&SideCarIdentifier);

			return Result;
		}

		LastError = Some(Result.unwrap_err());

		if Attempt < MAX_RETRY_ATTEMPTS {
			let DelayMilliseconds = RETRY_BASE_DELAY_MS * 2_u64.pow(Attempt as u32);

			tokio::time::sleep(Duration::from_millis(DelayMilliseconds)).await;
		}
	}

	Err(LastError.unwrap_or_else(|| VineError::RPCError("Connection failed".to_string())))
}
