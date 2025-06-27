//! # IPCProvider Implementation
//!
//! Implements the `IPCProvider` trait for the `MountainEnvironment`. This
//! provider serves as a simple bridge, delegating all IPC operations directly
//! to the `Vine` gRPC client.

use Common::{Error::CommonError::CommonError, IPC::IPCProvider::IPCProvider};
use async_trait::async_trait;
use serde_json::Value;

use super::MountainEnvironment::MountainEnvironment;
use crate::Vine::Client;

#[async_trait]
impl IPCProvider for MountainEnvironment {
	/// Sends a fire-and-forget notification to a specified sidecar.
	async fn SendNotificationToSideCar(
		&self,

		SideCarIdentifier:String,

		Method:String,

		Parameters:Value,
	) -> Result<(), CommonError> {
		Client::SendNotification(SideCarIdentifier, Method, Parameters)
			.await
			.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })
	}

	/// Sends a request to a specified sidecar and awaits a response.
	async fn SendRequestToSideCar(
		&self,

		SideCarIdentifier:String,

		Method:String,

		Parameters:Value,

		TimeoutMilliseconds:u64,
	) -> Result<Value, CommonError> {
		Client::SendRequest(&SideCarIdentifier, Method, Parameters, TimeoutMilliseconds)
			.await
			.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })
	}
}
