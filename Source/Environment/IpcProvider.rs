// File: Environment/IpcProvider.rs
// Implements the `IpcProvider` trait for the `MountainEnvironment`.
// This file connects abstract IPC effects to the concrete gRPC implementation
// in the `Vine` module.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Environment::Requires, Errors::CommonError, IpcEffect::IpcProvider};
use async_trait::async_trait;
use log::trace;
use serde_json::Value;

use crate::{Environment::MountainEnvironment, Vine}; // The gRPC communication module

#[async_trait]
impl IpcProvider for MountainEnvironment {
	/// Sends a fire-and-forget notification to a sidecar via gRPC.
	async fn SendNotificationToSidecar(
		&self,
		SidecarIdentifier:String,
		Method:String,
		Parameters:Value,
	) -> Result<(), CommonError> {
		trace!(
			"[Environment IpcProvider] SendNotification: Sidecar='{}', Method='{}'",
			SidecarIdentifier, Method
		);
		Vine::SendNotification(SidecarIdentifier, Method, Parameters)
			.await
			.map_err(|VineError| CommonError::IpcError { Description:VineError.to_string() })
	}

	/// Sends a request to a sidecar via gRPC and awaits a response.
	async fn SendRequestToSidecar(
		&self,
		SidecarIdentifier:String,
		Method:String,
		Parameters:Value,
		TimeoutMilliseconds:u64,
	) -> Result<Value, CommonError> {
		trace!(
			"[Environment IpcProvider] SendRequest: Sidecar='{}', Method='{}', Timeout={}ms",
			SidecarIdentifier, Method, TimeoutMilliseconds
		);
		// The request identifier is handled internally by the Vine gRPC client logic.
		let RequestIdentifier = self.GetAppState().GetNextTerminalIdentifier(); // Reusing this atomic counter
		Vine::SendRequest(SidecarIdentifier, Method, Parameters, TimeoutMilliseconds)
			.await
			.map_err(|VineError| CommonError::IpcError { Description:VineError.to_string() })
	}
}

impl Requires<Arc<dyn IpcProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn IpcProvider + Send + Sync> { Arc::new(self.clone()) }
}
