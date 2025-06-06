// ---------------------------------------------------------------------------------------------
// Mountain Environment - IPC Provider 
// --------------------------------------------------------------------------------------------
// This module implements the `IpcProvider` trait for `MountainEnvironment`.
// It provides a standardized way for other parts of Mountain (primarily effects
// or direct handlers like those for language features) to send notifications
// and requests to sidecar processes (e.g., Cocoon).
//
// The actual IPC mechanism (message serialization, transport, routing to the
// correct sidecar process) is handled by the `crate::vine` module. This
// provider acts as a simple facade over `vine`.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	environment::Requires,
	errors::CommonError,
	ipc_effects::IpcProvider, // The trait being implemented
};
use async_trait::async_trait;
use log::trace; // For logging
use serde_json::Value;

use crate::{
	environment::MountainEnvironment,
	vine, // For actual IPC communication
};

// --- IpcProvider Implementation ---
#[async_trait]
impl IpcProvider for MountainEnvironment {
	async fn send_notification_to_sidecar(
		&self,
		sidecar_id:String,
		method:String,
		params:Value,
	) -> Result<(), CommonError> {
		trace!(
			"[Env IpcProv] Sending notification to sidecar '{}': method='{}', params_type='{:?}'",
			sidecar_id,
			method,
			params.kind()
		);

		// Delegate directly to the Vine IPC layer.
		vine::send_notification_to_sidecar(&sidecar_id, method, params)
			.await
			.map_err(|vine_err| {
				// Map VineError (or whatever error type Vine returns) to CommonError::IpcError
				CommonError::IpcError(vine_err.to_string())
			})
	}

	async fn send_request_to_sidecar(
		&self,
		sidecar_id:String,
		method:String,
		params:Value,
		timeout_ms:u64,
	) -> Result<Value, CommonError> {
		trace!(
			"[Env IpcProv] Sending request to sidecar '{}': method='{}', params_type='{:?}', timeout_ms={}",
			sidecar_id,
			method,
			params.kind(),
			timeout_ms
		);

		// Delegate directly to the Vine IPC layer.
		vine::send_request_to_sidecar(&sidecar_id, method, params, timeout_ms)
			.await
			.map_err(|vine_err| CommonError::IpcError(vine_err.to_string()))
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn IpcProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn IpcProvider + Send + Sync> { Arc::new(self.clone()) }
}
