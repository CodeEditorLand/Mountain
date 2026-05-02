//! # IPCProvider (Environment)
//!
//! RESPONSIBILITIES:
//! - Implements [`IPCProvider`](CommonLibrary::IPC::IPCProvider) for
//!   [`MountainEnvironment`]
//! - Serves as the IPC bridge between Mountain and extension sidecar processes
//!   (Cocoon)
//! - Delegates all communications to the Vine gRPC client for transport
//! - Provides both request/response and notification patterns
//!
//! ARCHITECTURAL ROLE:
//! - Thin wrapper layer over `Vine::Client`
//! - All IPC operations are async and use JSON-RPC 2.0 protocol
//! - Sidecar routing via `SideCarIdentifier` to target specific extension hosts
//! - Error translation from Vine errors to
//!   [`CommonError::IPCError`](CommonLibrary::Error::CommonError)
//!
//! COMMUNICATION PATTERNS:
//! - **Request/Response** `SendRequestToSideCar`:
//! - Synchronous RPC with timeout protection
//! - Returns `Result<Value, CommonError>`
//! - Used for config resolution, URI lookup, content retrieval
//! - **Notification** `SendNotificationToSideCar`:
//! - Fire-and-forget pattern
//! - Returns `Result<(), CommonError>` (indicating send success only)
//! - Used for document changes, diagnostics, UI events
//!
//! PERFORMANCE:
//! - Vine client manages connection pooling and reuse
//! - Timeouts enforced on requests to prevent blocking (caller-specified)
//! - TODO: Add request queuing, batching, and priority handling
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/services/extensions/common/extensionHostProtocol.ts` -
//!   main-ext host IPC
//! - `vs/base/parts/ipc/common/ipc.net.ts` - IPC transport layer
//! - `vs/workbench/services/extensions/common/rpcProtocol.ts` - JSON-RPC
//!   protocol
//!
//! TODO:
//! - Add message queuing for offline scenarios (caching when sidecar is down)
//! - Implement bidirectional request handling (sidecar → main process)
//! - Add request/response streaming support for large data transfers
//! - Implement request cancellation with token support
//! - Add request metrics and telemetry (latency, success rate, etc.)
//! - Implement priority queue for urgent messages
//! - Add support for batch IPC operations
//! - Consider adding request deduplication
//! - Implement proper connection health checking
//! - Add support for IPC over Unix domain sockets for local sidecars
//!
//! MODULE CONTENTS:
//! - [`IPCProvider`](CommonLibrary::IPC::IPCProvider) implementation:
//! - `SendNotificationToSideCar` - fire-and-forget
//! - `SendRequestToSideCar` - synchronous RPC
//! - Delegate: `Vine::Client` handles all transport concerns

use CommonLibrary::{Error::CommonError::CommonError, IPC::IPCProvider::IPCProvider};
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
