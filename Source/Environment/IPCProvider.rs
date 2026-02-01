// File: Mountain/Source/Environment/IPCProvider.rs
//
// # Architectural Role: Inter-Process Communication Bridge
//
// IPCProvider implements the IPCProvider trait, serving as the communication
// bridge between Mountain (main process) and extension sidecar processes
// (Cocoon). It delegates all IPC operations to the Vine gRPC client, which
// manages the actual transport layer and connection handling.
//
// # Responsibilities
//
// 1. **Request/Response Communication**: Handles synchronous RPC requests where
//    Mountain sends a request and awaits a response from a sidecar.
//
// 2. **Notification Communication**: Handles fire-and-forget notifications
//    where Mountain sends messages to sidecars without waiting for responses.
//
// 3. **Sidecar Routing**: Routes all IPC messages to the appropriate sidecar
//    process via the Vine client.
//
// 4. **Timeout Management**: Enforces timeout limits for request/response
//    operations to prevent indefinite blocking.
//
// 5. **Error Translation**: Converts Vine client errors into CommonError types
//    for consistent error handling across the application.
//
// # Communication Patterns
//
// **Request/Response** (SendRequestToSideCar):
// - Used for operations that require a return value
// - Examples: Getting configuration, resolving URIs, retrieving content
// - Timeout enforced to prevent hanging
// - Returns Result<Value, CommonError>
//
// **Notification** (SendNotificationToSideCar):
// - Used for events that don't require responses
// - Examples: Document changes, diagnostics updates, user interactions
// - Fire-and-forget, returns only success/failure
// - More efficient for high-frequency events
//
// # Vine Client Integration
//
// The IPCProvider delegates all operations to the Vine gRPC client, which:
// - Manages connections to all sidecar processes
// - Implements message serialization/deserialization
// - Handles connection recovery and retries
// - Provides load balancing for multiple sidecar instances
// - Implements circuit breaker pattern for failures
//
// # TODOs
//
// - [ ] Add message queuing for offline scenarios (caching messages when
//   sidecar is down)
// - [ ] Implement bidirectional request handling (sidecar → main process)
// - [ ] Add request/response streaming support for large data transfers
// - [ ] Implement request cancellation with token support
// - [ ] Add request metrics and telemetry (latency, success rate, etc.)
// - [ ] Implement priority queue for urgent messages
// - [ ] Add support for batch IPC operations
// - [ ] Consider adding request deduplication
// - [ ] Implement proper connection health checking
// - [ ] Add support for IPC over Unix domain sockets for local sidecars
//
// # Patterns Borrowed from VSCode
//
// - **RPC Bridge**: Similar to VSCode's RPC protocol between main process and
//   extension host.
//
// - **JSON-RPC**: Uses JSON-RPC 2.0 protocol for message format, like VSCode.
//
// - **Message Targeting**: Employs ProxyTarget pattern for routing messages to
//   specific extension hosts, similar to VSCode's architecture.

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
