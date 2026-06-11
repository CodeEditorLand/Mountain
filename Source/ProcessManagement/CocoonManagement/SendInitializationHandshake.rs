//! Send the `InitializeExtensionHost` payload to a freshly connected Cocoon
//! and validate the `"initialized"` handshake response.

use std::{sync::Arc, time::Duration};

use CommonLibrary::Error::CommonError::CommonError;
use tokio::time::sleep;

use crate::{Environment::MountainEnvironment::MountainEnvironment, ProcessManagement::InitializationData, dev_log};

pub(crate) async fn Fn(SideCarIdentifier:&str, Environment:&Arc<MountainEnvironment>) -> Result<(), CommonError> {
	dev_log!(
		"cocoon",
		"[CocoonManagement] Connected to Cocoon. Sending initialization data..."
	);

	// Brief delay to ensure Cocoon's gRPC service handlers are fully registered
	// after bindAsync resolves (race condition on fast connections like attempt 1)
	sleep(Duration::from_millis(200)).await;

	// Construct initialization payload
	let MainInitializationData = InitializationData::ConstructExtensionHostInitializationData(Environment)
		.await
		.map_err(|Error| {
			CommonError::IPCError { Description:format!("Failed to construct initialization data: {}", Error) }
		})?;

	// Send initialization request with timeout
	let Response = crate::Vine::Client::SendRequest::Fn(
		SideCarIdentifier,
		"InitializeExtensionHost".to_string(),
		MainInitializationData,
		super::HANDSHAKE_TIMEOUT_MS,
	)
	.await
	.map_err(|Error| {
		CommonError::IPCError {
			Description:format!("Failed to send initialization request to Cocoon: {}", Error),
		}
	})?;

	// Validate handshake response
	match Response.as_str() {
		Some("initialized") => {
			dev_log!(
				"cocoon",
				"[CocoonManagement] Cocoon handshake complete. Extension host is ready."
			);
		},

		Some(other) => {
			return Err(CommonError::IPCError {
				Description:format!("Cocoon initialization failed with unexpected response: {}", other),
			});
		},

		None => {
			return Err(CommonError::IPCError {
				Description:"Cocoon initialization failed: no response received".to_string(),
			});
		},
	}

	Ok(())
}
