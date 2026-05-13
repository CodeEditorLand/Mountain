//! # IPCProvider (Environment)
//!
//! Implements [`IPCProvider`](CommonLibrary::IPC::IPCProvider) for
//! `MountainEnvironment`. Serves as the IPC bridge between Mountain and
//! extension sidecar processes (Cocoon), delegating all transport to the
//! Vine gRPC client with JSON-RPC 2.0 over the wire.
//!
//! ## Communication patterns
//!
//! - **Request/response** (`SendRequestToSideCar`) - synchronous RPC with
//!   caller-specified timeout; used for config resolution, URI lookup, and
//!   content retrieval.
//! - **Notification** (`SendNotificationToSideCar`) - fire-and-forget; used
//!   for document changes, diagnostics, and UI events. Returns
//!   `Result<(), CommonError>` indicating send success only.
//!
//! ## VS Code reference
//!
//! - `vs/workbench/services/extensions/common/extensionHostProtocol.ts`
//! - `vs/base/parts/ipc/common/ipc.net.ts`
//! - `vs/workbench/services/extensions/common/rpcProtocol.ts`
//!
//! ## Planned Work
//!
//! - Message queuing for offline scenarios
//! - Bidirectional request handling (sidecar → main)
//! - Streaming support
//! - Request cancellation
//! - Priority queue and batch operations
//! - Request deduplication
//! - Connection health checking
//! - Unix domain socket support
//! - Latency/success-rate telemetry

use CommonLibrary::{Error::CommonError::CommonError, IPC::IPCProvider::IPCProvider};
use async_trait::async_trait;
use serde_json::Value;

use super::MountainEnvironment::MountainEnvironment;
use crate::Vine::Client;

// TODO: message queuing for offline scenarios, bidirectional request handling
// (sidecar → main), streaming support, request cancellation, priority queue,
// batch operations, request deduplication, connection health checking,
// Unix domain socket support, latency/success-rate telemetry.
#[async_trait]
impl IPCProvider for MountainEnvironment {
	/// Sends a fire-and-forget notification to a specified sidecar.
	async fn SendNotificationToSideCar(
		&self,

		SideCarIdentifier:String,

		Method:String,

		Parameters:Value,
	) -> Result<(), CommonError> {
		Client::SendNotification::Fn(SideCarIdentifier, Method, Parameters)
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
		Client::SendRequest::Fn(&SideCarIdentifier, Method, Parameters, TimeoutMilliseconds)
			.await
			.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })
	}
}
