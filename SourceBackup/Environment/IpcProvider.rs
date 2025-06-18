// @module IpcProvider (Environment)
// @description Implements the `IpcProvider` trait for `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{Environment::Requires, error::CommonError, IPC::IpcProvider};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Vine::client;

#[async_trait]
impl IpcProvider for MountainEnvironment {
	// Sends a notification (fire-and-forget message) to a specified sidecar.
	async fn SendNotificationToSidecar(
		&self,
		sidecar_identifier:String,
		method:String,
		parameters:Value,
	) -> Result<(), CommonError> {
		client::SendNotification(sidecar_identifier, method, parameters).await
	}

	// Sends a request to a specified sidecar and awaits a response.
	async fn SendRequestToSidecar(
		&self,
		sidecar_identifier:String,
		method:String,
		parameters:Value,
		timeout_milliseconds:u64,
	) -> Result<Value, CommonError> {
		client::SendRequest(sidecar_identifier, method, parameters, timeout_milliseconds).await
	}
}

impl Requires<Arc<dyn IpcProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn IpcProvider + Send + Sync> { Arc::new(self.clone()) }
}
