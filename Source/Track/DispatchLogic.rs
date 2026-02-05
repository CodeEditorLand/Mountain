//! # DispatchLogic (Track)
//!
//! Contains the main dispatch functions for routing all incoming commands and
//! RPC requests to the appropriate execution logic via the effect system.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Request Routing
//! - Route all incoming requests from the Sky frontend to appropriate effects
//! - Route all incoming requests from Cocoon sidecars to appropriate effects
//! - Provide specific, high-performance Tauri command handlers for frequent UI interactions
//! - Handle both synchronous (UI) and asynchronous (backend) dispatch paths
//!
//! ### 2. Error Handling and Recovery
//! - Provide robust error handling for all dispatch operations
//! - Log all routing decisions for observability and debugging
//! - Handle effect creation failures gracefully
//! - Support error context propagation to callers
//!
//! ### 3. Type Safety and Validation
//! - Ensure type-safe message passing through Rust's type system
//! - Validate request parameters before dispatch
//! - Handle serialization/deserialization errors
//! - Support complex parameter types (JSON, URLs, etc.)
//!
//! ## ARCHITECTURAL ROLE
//!
//! DispatchLogic is the **dispatcher layer** that sits between request sources
//! and the effect execution system:
//!
//! ```text
//! Sky (Frontend) ──► DispatchLogic ──► EffectCreation ──► ApplicationRunTime ──► Providers
//! Cocoon (Sidecar) ──► DispatchLogic ──► EffectCreation ──► ApplicationRunTime ──► Providers
//! ```
//!
//! ### Design Patterns:
//! 1. **Command Pattern**: Execute commands by name
//! 2. **Strategy Pattern**: Different dispatch strategies for frontend vs sidecar
//! 3. **Observer Pattern**: Handle UI request-response interactions
//!
//! ### VS Code Reference:
//! This module borrows from VS Code's request routing:
//! - `vs/workbench/services/extensions/common/rpcProtocol.ts` - RPC routing
//! - `vs/base/parts/ipc/common/ipc.ts` - IPC message dispatch
//! - `vs/platform/request/common/request.ts` - Request handling
//!
//! Key concepts from VS Code:
//! - Async request/response pairs with unique IDs
//! - Type-safe message passing
//! - Comprehensive error propagation
//! - Request cancellation support (TODO)
//!
//! ## KEY COMPONENTS
//!
//! ### DispatchFrontendCommand
//! The primary Tauri command handler for requests originating from the Sky
//! frontend. General-purpose entry point for commands defined in Common crate.
//!
//! **Flow**:
//! 1. Log the incoming command
//! 2. Call `EffectCreation::CreateEffectForRequest` to create effect
//! 3. Execute effect with runtime
//! 4. Return result or error to frontend
//!
//! ### DispatchSideCarRequest
//! Dispatcher for requests from Cocoon sidecars via gRPC. Routes RPC calls to
//! effect-based implementations.
//!
//! **Flow**:
//! 1. Log incoming sidecar request with sidecar ID
//! 2. Create effect via `EffectCreation::CreateEffectForRequest`
//! 3. Execute effect with runtime
//! 4. Return result or error to sidecar
//!
//! ### ResolveUIRequest
//! Tauri command handler for UI to send back results of request-response
//! interactions (dialogs, message boxes).
//!
//! **Flow**:
//! 1. Look up pending UI request by ID
//! 2. Remove from pending requests map
//! 3. Send result through oneshot channel
//! 4. Handle dropped receiver errors gracefully
//!
//! ### MountainWebviewPostMessageFromGuest
//! Tauri command handler for Webview guest to post message back to extension host.
//!
//! **Flow**:
//! 1. Get IPC provider from runtime
//! 2. Forward message to main Cocoon sidecar
//! 3. Handle IPC errors gracefully
//!
//! ## ERROR HANDLING
//!
//! All dispatch functions return `Result<Value, String>` where:
//! - `Ok(Value)`: Successful execution with JSON-serializable result
//! - `Err(String)`: Error with descriptive message
//!
//! Error recovery mechanisms:
//! - Effect creation failures are caught and logged
//! - Unknown commands are reported with context
//! - UI request timeouts are handled gracefully
//! - IPC errors are propagated with context
//!
//! ## LOGGING
//!
//! Comprehensive logging for:
//! - All incoming commands and RPC requests
//! - Effect creation failures
//! - UI request resolution
//! - Webview message forwarding
//! - All errors with context
//!
//! Log levels:
//! - `debug`: Normal operation flow
//! - `info`: Significant events
//! - `warn`: Recoverable issues
//! - `error`: Critical failures
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Direct provider calls used for hot paths (configuration, diagnostics)
//! - Effect-based execution for consistency and maintainability
//! - Async operations to avoid blocking UI thread
//! - Minimal locking and contention
//!
//! ## REQUEST FLOW EXAMPLE
//!
//! ```text
//! Frontend: app.invoke("vscode.getConfiguration", { section: "editor" })
//!    ↓
//! Tauri: DispatchFrontendCommand(Command="vscode.getConfiguration", ...)
//!    ↓
//! EffectCreation::CreateEffectForRequest() ──► Lookup effect builder
//!    ↓
//! ApplicationRunTime::Run(Effect) ──► Submit to Echo scheduler
//!    ↓
//! Effect::Apply(capability) ──► Call ConfigurationProvider::GetConfiguration
//!    ↓
//! Result serialized to JSON ──► Returned to frontend
//! ```
//!
//! ## TODO
//!
//! High Priority:
//! - [ ] Add request timeout handling
//! - [ ] Implement request cancellation support
//! - [ ] Add request metrics and telemetry
//!
//! Medium Priority:
//! - [ ] Add request throttling for high-frequency operations
//! - [ ] Implement request batching for related operations
//! - [ ] Add request replay for debugging
//!
//! Low Priority:
//! - [ ] Add request caching for idempotent operations
//! - [ ] Implement request tracing across entire pipeline
//! - [ ] Add request priority queuing
//!
//! ## MODULE CONTENTS
//!
//! - [`DispatchFrontendCommand`]: Main frontend command handler (Tauri command)
//! - [`DispatchSideCarRequest`]: Sidecar RPC request dispatcher
//! - [`ResolveUIRequest`]: UI request-response result handler (Tauri command)
//! - [`MountainWebviewPostMessageFromGuest`]: Webview message forwarder (Tauri command)

//
// This module provides the main dispatch functions for routing all incoming
// commands and RPC requests to the appropriate execution logic via the effect
// system. Its primary responsibilities are:
//
// ### 1. Request Routing
// - Route all incoming requests from the Sky frontend to appropriate effects
// - Route all incoming requests from Cocoon sidecars to appropriate effects
// - Provide specific, high-performance Tauri command handlers for frequent UI
//   interactions
// - Handle both synchronous (UI) and asynchronous (backend) dispatch paths
//
// ### 2. Error Handling and Recovery
// - Provide robust error handling for all dispatch operations
// - Log all routing decisions for observability and debugging
// - Handle effect creation failures gracefully
// - Support error context propagation to callers
//
// ### 3. Type Safety and Validation
// - Ensure type-safe message passing through Rust's type system
// - Validate request parameters before dispatch
// - Handle serialization/deserialization errors
// - Support complex parameter types (JSON, URLs, etc.)
//
// ## Architectural Role
//
// DispatchLogic is the **dispatcher layer** that sits between:
//
// Sky (Frontend) ──► DispatchLogic (Dispatcher) ──► EffectCreation (Mapper) ──►
// ApplicationRunTime (Executor) Cocoon (Sidecar) ──► DispatchLogic (Dispatcher)
// ──► EffectCreation (Mapper) ──► ApplicationRunTime (Executor)
//
// ### Design Patterns:
// 1. **Command Pattern**: Execute commands by name
// 2. **Strategy Pattern**: Different dispatch strategies for frontend vs
//    sidecar
// 3. **Observer Pattern**: Handle UI request-response interactions
//
// ### VS Code Reference:
// This module borrows from VS Code's request routing in:
// - `vs/workbench/services/extensions/common/rpcProtocol.ts` - RPC routing
// - `vs/base/parts/ipc/common/ipc.ts` - IPC message dispatch
// - `vs/platform/request/common/request.ts` - Request handling
//
// Key concepts from VS Code:
// - Async request/response pairs with unique IDs
// - Type-safe message passing
// - Comprehensive error propagation
// - Request cancellation support
//
// ## Key Components
//
// ### DispatchFrontendCommand
// The primary Tauri command handler for requests originating from the Sky
// frontend. This is the general-purpose entry point for commands that are
// defined abstractly in the `Common` crate.
//
// **Flow:**
// 1. Log the incoming command
// 2. Call CreateEffectForRequest to create the effect
// 3. Execute the effect with the runtime
// 4. Return the result or error to the frontend
//
// ### DispatchSideCarRequest
// The primary dispatcher for requests originating from a Cocoon sidecar via
// gRPC. This routes RPC calls to the correct effect-based implementation.
//
// **Flow:**
// 1. Log the incoming sidecar request with sidecar ID
// 2. Call CreateEffectForRequest to create the effect
// 3. Execute the effect with the runtime
// 4. Return the result or error to the sidecar
//
// ### ResolveUIRequest
// A specific Tauri command handler for the UI to send back the result of a
// request-response interaction (like a dialog or message box).
//
// **Flow:**
// 1. Look up the pending UI request by ID
// 2. Remove it from the pending requests map
// 3. Send the result through the oneshot channel
// 4. Handle dropped receiver errors gracefully
//
// ### MountainWebviewPostMessageFromGuest
// A specific Tauri command handler for a Webview guest to post a message back
// to the extension host.
//
// **Flow:**
// 1. Get the IPC provider from the runtime
// 2. Forward the message to the main Cocoon sidecar
// 3. Handle IPC errors gracefully
//
// ## Error Handling
//
// All dispatch functions return `Result<Value, String>` where:
// - `Ok(Value)` - Successful execution with JSON-serializable result
// - `Err(String)` - Error with descriptive message
//
// Error recovery mechanisms:
// - Effect creation failures are caught and logged
// - Unknown commands are reported with context
// - UI request timeouts are handled gracefully
// - IPC errors are propagated with context
//
// ## Logging
//
// Comprehensive logging is provided for:
// - All incoming commands and RPC requests
// - Effect creation failures
// - UI request resolution
// - Webview message forwarding
// - All errors with context
//
// Log levels:
// - `debug` - Normal operation flow
// - `info` - Significant events
// - `warn` - Recoverable issues
// - `error` - Critical failures
//
// ## Performance Considerations
//
// - Direct provider calls are used for hot paths (configuration, diagnostics)
// - Effect-based execution for consistency and maintainability
// - Async operations to avoid blocking the UI thread
// - Minimal locking and contention
//
// ## TODOs
//
// High Priority:
// - [ ] Add request timeout handling
// - [ ] Implement request cancellation support
// - [ ] Add request metrics and telemetry
//
// Medium Priority:
// - [ ] Add request throttling for high-frequency operations
// - [ ] Implement request batching for related operations
// - [ ] Add request replay for debugging
//
// Low Priority:
// - [ ] Add request caching for idempotent operations
// - [ ] Implement request tracing across the entire pipeline
// - [ ] Add request priority queuing

//! # DispatchLogic
//!
//! Contains the main dispatch functions for routing all incoming commands and
//! RPC requests to the appropriate execution logic via the effect system.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, IPC::IPCProvider::IPCProvider};
use log::{debug, error, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, State, command};

use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::EffectCreation,
};

/// The primary Tauri command handler for requests originating from the `Sky`
/// frontend. This is the general-purpose entry point for commands that are
/// defined abstractly in the `Common` crate.
#[command]
pub async fn DispatchFrontendCommand<R:Runtime>(
	ApplicationHandle:AppHandle<R>,

	RunTime:State<'_, Arc<ApplicationRunTime>>,

	Command:String,

	Argument:Value,
) -> Result<Value, String> {
	debug!("[DispatchLogic] Dispatching frontend command: {}", Command);

	match EffectCreation::CreateEffectForRequest(&ApplicationHandle, &Command, Argument) {
		Ok(EffectFn) => {
			let runtime_clone = RunTime.inner().clone();

			EffectFn(runtime_clone).await
		},

		Err(Error) => {
			error!("[DispatchLogic] Failed to create effect for command '{}': {}", Command, Error);

			Err(Error)
		},
	}
}

/// The primary dispatcher for requests originating from a `Cocoon` sidecar via
/// gRPC. This routes RPC calls to the correct effect-based implementation.
pub async fn DispatchSideCarRequest<R:Runtime>(
	ApplicationHandle:AppHandle<R>,

	RunTime:Arc<ApplicationRunTime>,

	SideCarIdentifier:String,

	MethodName:String,

	Parameters:Value,
) -> Result<Value, String> {
	debug!(
		"[DispatchLogic] Dispatching sidecar request from '{}': {}",
		SideCarIdentifier, MethodName
	);

	match EffectCreation::CreateEffectForRequest(&ApplicationHandle, &MethodName, Parameters) {
		Ok(EffectFn) => EffectFn(RunTime).await,

		Err(Error) => {
			error!(
				"[DispatchLogic] Failed to create effect for sidecar method '{}': {}",
				MethodName, Error
			);

			Err(Error)
		},
	}
}

/// A specific Tauri command handler for the UI to send back the result of a
/// request-response interaction (like a dialog or message box).
#[command]
pub async fn ResolveUIRequest(
	State:State<'_, Arc<ApplicationState>>,

	RequestID:String,

	Result:Value,
) -> Result<(), String> {
	debug!("[DispatchLogic] Resolving UI request ID: {}", RequestID);

	let Sender = {
		let mut PendingRequests = State.PendingUserInterfaceRequests.lock().map_err(|Error| Error.to_string())?;

		PendingRequests.remove(&RequestID)
	};

	if let Some(Sender) = Sender {
		if Sender.send(Ok(Result)).is_err() {
			let ErrorMessage = format!("Failed to send result for UI request '{}': receiver was dropped.", RequestID);

			error!("{}", ErrorMessage);

			return Err(ErrorMessage);
		}
	} else {
		warn!(
			"[DispatchLogic] Received a result for an unknown or timed-out UI request ID: {}",
			RequestID
		);
	}

	Ok(())
}

/// A specific Tauri command handler for a Webview guest to post a message back
/// to the extension host.
#[command]
pub async fn MountainWebviewPostMessageFromGuest(
	ApplicationHandle:AppHandle,

	Handle:String,

	Message:Value,
) -> Result<(), String> {
	let IPC:Arc<dyn IPCProvider> = {
		let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		RunTime.Environment.Require()
	};

	let RPCResult = IPC
		.SendNotificationToSideCar("cocoon-main".into(), "$onDidReceiveMessage".into(), json!([Handle, Message]))
		.await;

	if let Err(Error) = RPCResult {
		error!("[DispatchLogic] Failed to forward webview message to Cocoon: {}", Error);

		return Err(Error.to_string());
	}

	Ok(())
}
