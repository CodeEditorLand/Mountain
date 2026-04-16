//! # MountainVinegRPCService
//!
//! Defines the gRPC service implementation for Mountain. This struct handles
//! incoming RPC calls from the `Cocoon` sidecar, dispatches them to the
//! application's core logic via the `Track` module, and returns the results.
//!
//! ## Service Methods
//!
//! - **process_cocoon_request**: Handles request-response calls from Cocoon
//! - **send_cocoon_notification**: Handles fire-and-forget notifications from
//!   Cocoon
//! - **cancel_operation**: Cancels long-running operations requested by Cocoon
//!
//! ## Request Processing
//!
//! 1. Deserialize JSON parameters from request
//! 2. Validate method name and parameters
//! 3. Dispatch request to Track::DispatchLogic
//! 4. Serialize response or error
//! 5. Return gRPC response with proper status codes
//!
//! ## Error Handling
//!
//! All errors are converted to JSON-RPC compliant Error objects:
//! - Parse errors: code -32700
//! - Server errors: code -32000
//! - Method not found: code -32601
//! - Invalid params: code -32602
//!
//! ## Security
//!
//! - Parameter validation before processing
//! - Message size limits enforced
//! - Method name sanitization
//! - Safe error messages (no sensitive data)

use std::{collections::HashMap, sync::Arc};

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::dev_log;
use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track,
	Vine::Generated::{
		CancelOperationRequest,
		Empty,
		GenericNotification,
		GenericRequest,
		GenericResponse,
		RpcError as RPCError,
		mountain_service_server::MountainService,
	},
};

/// Configuration for MountainService
mod ServiceConfig {
	/// Maximum number of concurrent operations
	pub const MAX_CONCURRENT_OPERATIONS:usize = 50;

	/// Default timeout for operation cancellation
	pub const CANCELLATION_TIMEOUT_MS:u64 = 5000;

	/// Maximum method name length
	pub const MAX_METHOD_NAME_LENGTH:usize = 128;
}

/// The concrete implementation of the `MountainService` gRPC service.
///
/// This service handles all incoming RPC calls from the Cocoon sidecar,
/// validating requests, dispatching to appropriate handlers, and returning
/// responses in the expected gRPC format.
pub struct MountainVinegRPCService {
	/// Tauri application handle for VS Code integration
	ApplicationHandle:AppHandle,

	/// Application runtime containing core dependencies
	RunTime:Arc<ApplicationRunTime>,

	/// Registry of active operations with their cancellation tokens
	/// Maps request ID to cancellation token for operation cancellation
	ActiveOperations:Arc<RwLock<HashMap<u64, tokio_util::sync::CancellationToken>>>,
}

impl MountainVinegRPCService {
	/// Creates a new instance of the Mountain gRPC service.
	///
	/// # Parameters
	/// - `ApplicationHandle`: Tauri app handle for framework integration
	/// - `RunTime`: Application runtime with core dependencies
	///
	/// # Returns
	/// New MountainVinegRPCService instance
	pub fn Create(ApplicationHandle:AppHandle, RunTime:Arc<ApplicationRunTime>) -> Self {
		dev_log!("grpc", "[MountainVinegRPCService] New instance created");

		Self {
			ApplicationHandle,
			RunTime,
			ActiveOperations:Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// Registers an operation for potential cancellation
	///
	/// # Parameters
	/// - `request_id`: The request identifier for the operation
	///
	/// # Returns
	/// A cancellation token that can be used to cancel the operation
	pub async fn RegisterOperation(&self, request_id:u64) -> tokio_util::sync::CancellationToken {
		let token = tokio_util::sync::CancellationToken::new();
		self.ActiveOperations.write().await.insert(request_id, token.clone());
		dev_log!("grpc", "[MountainVinegRPCService] Registered operation {} for cancellation", request_id);
		token
	}

	/// Unregisters an operation after completion
	///
	/// # Parameters
	/// - `request_id`: The request identifier to unregister
	pub async fn UnregisterOperation(&self, request_id:u64) {
		self.ActiveOperations.write().await.remove(&request_id);
		dev_log!("grpc", "[MountainVinegRPCService] Unregistered operation {}", request_id);
	}

	/// Validates a generic request before processing.
	///
	/// # Parameters
	/// - `request`: The request to validate
	///
	/// # Returns
	/// - `Ok(())`: Request is valid
	/// - `Err(Status)`: Validation failed with appropriate gRPC status
	fn ValidateRequest(&self, request:&GenericRequest) -> Result<(), Status> {
		// Validate method name
		if request.method.is_empty() {
			return Err(Status::invalid_argument("Method name cannot be empty"));
		}

		if request.method.len() > ServiceConfig::MAX_METHOD_NAME_LENGTH {
			return Err(Status::invalid_argument(format!(
				"Method name exceeds maximum length of {} characters",
				ServiceConfig::MAX_METHOD_NAME_LENGTH
			)));
		}

		// Validate parameter size (rough estimate using JSON bytes)
		if request.parameter.len() > 4 * 1024 * 1024 {
			return Err(Status::resource_exhausted("Request parameter size exceeds limit"));
		}

		// Check for potentially malicious method names
		if request.method.contains("../") || request.method.contains("::") {
			return Err(Status::permission_denied("Invalid method name format"));
		}

		Ok(())
	}

	/// Creates a JSON-RPC compliant error response.
	///
	/// # Parameters
	/// - `RequestIdentifier`: The request ID to echo back
	/// - `code`: JSON-RPC error code
	/// - `message`: Error message
	/// - `data`: Optional error data (serialized)
	///
	/// # Returns
	/// GenericResponse with error populated
	fn CreateErrorResponse(RequestIdentifier:u64, code:i32, message:String, data:Option<Vec<u8>>) -> GenericResponse {
		GenericResponse {
			request_identifier:RequestIdentifier,
			result:vec![],
			error:Some(RPCError { code, message, data:data.unwrap_or_default() }),
		}
	}

	/// Creates a successful JSON-RPC response.
	///
	/// # Parameters
	/// - `RequestIdentifier`: The request ID to echo back
	/// - `result`: Result value to serialize
	///
	/// # Returns
	/// GenericResponse with result populated, or error if serialization fails
	fn CreateSuccessResponse(RequestIdentifier:u64, result:&Value) -> GenericResponse {
		let result_bytes = match serde_json::to_vec(result) {
			Ok(bytes) => bytes,
			Err(e) => {
				dev_log!("grpc", "error: [MountainVinegRPCService] Failed to serialize result: {}", e);

				// Return error response instead
				return Self::CreateErrorResponse(
					RequestIdentifier,
					-32603, // Internal error
					"Failed to serialize response".to_string(),
					None,
				);
			},
		};

		GenericResponse { request_identifier:RequestIdentifier, result:result_bytes, error:None }
	}
}

#[tonic::async_trait]
impl MountainService for MountainVinegRPCService {
	/// Handles generic request-response RPCs from Cocoon.
	///
	/// This is the main entry point for Cocoon to request operations from
	/// Mountain. It validates the request, deserializes parameters, dispatches
	/// to the Track module, and returns the result or error in JSON-RPC
	/// format.
	///
	/// # Parameters
	/// - `request`: GenericRequest containing method name and serialized
	///   parameters
	///
	/// # Returns
	/// - `Ok(Response<GenericResponse>)`: Response with result or error
	/// - `Err(Status)`: gRPC status error (only for critical failures)
	async fn process_cocoon_request(
		&self,
		request:Request<GenericRequest>,
	) -> Result<Response<GenericResponse>, Status> {
		let RequestData = request.into_inner();

		let MethodName = RequestData.method.clone();

		let RequestIdentifier = RequestData.request_identifier;

		dev_log!("grpc", 
			"[MountainVinegRPCService] Received gRPC Request [ID: {}]: Method='{}'",
			RequestIdentifier, MethodName
		);

		// Validate request before processing
		if let Err(status) = self.ValidateRequest(&RequestData) {
			dev_log!("grpc", "warn: [MountainVinegRPCService] Request validation failed: {}", status);

			return Ok(Response::new(Self::CreateErrorResponse(
				RequestIdentifier,
				-32602, // Invalid params
				status.message().to_string(),
				None,
			)));
		}

		// Deserialize JSON parameters
		let ParametersValue:Value = match serde_json::from_slice(&RequestData.parameter) {
			Ok(v) => {
				dev_log!("grpc", "[MountainVinegRPCService] Params for [ID: {}]: {:?}", RequestIdentifier, v);
				v
			},
			Err(e) => {
				let msg = format!("Failed to deserialize parameters for method '{}': {}", MethodName, e);

				dev_log!("grpc", "error: {}", msg);

				return Ok(Response::new(Self::CreateErrorResponse(
					RequestIdentifier,
					-32700, // Parse error
					msg,
					None,
				)));
			},
		};

		dev_log!("grpc", 
			"[MountainVinegRPCService] Dispatching request [ID: {}] to Track::DispatchLogic",
			RequestIdentifier
		);

		// Dispatch request to Track module for processing
		let DispatchResult = Track::SideCarRequest::DispatchSideCarRequest(
			self.ApplicationHandle.clone(),
			self.RunTime.clone(),
			// In the future, this could come from connection metadata
			"cocoon-main".to_string(),
			MethodName.clone(),
			ParametersValue,
		)
		.await;

		match DispatchResult {
			Ok(SuccessfulResult) => {
				dev_log!("grpc", 
					"[MountainVinegRPCService] Request [ID: {}] completed successfully",
					RequestIdentifier
				);

				Ok(Response::new(Self::CreateSuccessResponse(RequestIdentifier, &SuccessfulResult)))
			},

			Err(ErrorString) => {
				dev_log!("grpc", "error: [MountainVinegRPCService] Request [ID: {}] failed: {}",
					RequestIdentifier, ErrorString);

				Ok(Response::new(Self::CreateErrorResponse(
					RequestIdentifier,
					-32000, // Server error
					ErrorString,
					None,
				)))
			},
		}
	}

	/// Handles generic fire-and-forget notifications from Cocoon.
	///
	/// Notifications do not expect a response beyond acknowledgment.
	/// They are used for status updates, events, and other asynchronous
	/// notifications.
	///
	/// # Parameters
	/// - `request`: GenericNotification with method name and parameters
	///
	/// # Returns
	/// - `Ok(Response<Empty>)`: Notification was received and logged
	/// - `Err(Status)`: Critical error during processing
	///
	/// # TODO
	/// Future implementation should route notifications to dedicated handlers:
	/// ```rust,ignore
	/// let Parameter: Value = serde_json::from_slice(&notification.parameter)?;
	/// NotificationHandler::Handle(MethodName, Parameter).await?;
	/// ```
	async fn send_cocoon_notification(&self, request:Request<GenericNotification>) -> Result<Response<Empty>, Status> {
		let NotificationData = request.into_inner();

		let MethodName = NotificationData.method;

		dev_log!("grpc", "[MountainVinegRPCService] Received gRPC Notification: Method='{}'", MethodName);

		// Validate notification method name
		if MethodName.is_empty() {
			dev_log!("grpc", "warn: [MountainVinegRPCService] Received notification with empty method name");
			return Err(Status::invalid_argument("Method name cannot be empty"));
		}

		// Route notifications to appropriate handlers based on MethodName. Currently
		// only logs known notification types and acknowledges all others. A complete
		// implementation would maintain a registry of notification handlers per method,
		// route notifications to registered handlers asynchronously, allow handlers
		// to perform side effects (state updates, UI updates), support cancellation
		// and timeouts for long-running handlers, and log unhandled notifications
		// at debug level for diagnostics. Known notifications include:
		// ExtensionActivated, ExtensionDeactivated, WebviewReady.

		// Parse parameters for handlers that need them
		let Parameter:Value = if NotificationData.parameter.is_empty() {
			Value::Null
		} else {
			serde_json::from_slice(&NotificationData.parameter).unwrap_or(Value::Null)
		};

		match MethodName.as_str() {
			// Cocoon → Mountain → Wind: extension host binary protocol reply
			"extensionHostMessage" => {
				dev_log!("grpc", "[MountainVinegRPCService] Extension host message from Cocoon, forwarding to Wind");
				if let Err(Error) = self.ApplicationHandle.emit("cocoon:extensionHostReply", &Parameter) {
					dev_log!("grpc", "warn: [MountainVinegRPCService] Failed to emit cocoon:extensionHostReply: {}", Error);
				}
			},
			"ExtensionActivated" => {
				dev_log!("grpc", "[MountainVinegRPCService] Extension activated notification received");
				if let Err(Error) = self.ApplicationHandle.emit("cocoon:extensionActivated", &Parameter) {
					dev_log!("grpc", "warn: [MountainVinegRPCService] Failed to emit cocoon:extensionActivated: {}", Error);
				}
			},
			"ExtensionDeactivated" => {
				dev_log!("grpc", "[MountainVinegRPCService] Extension deactivated notification received");
			},
			"WebviewReady" => {
				dev_log!("grpc", "[MountainVinegRPCService] Webview ready notification received");
			},
			// Cocoon → Mountain → Sky: window messages (info/warn/error)
			"window.showMessage" => {
				dev_log!("grpc", "[MountainVinegRPCService] Window message from Cocoon: {:?}",
					Parameter.get("message").and_then(|m| m.as_str()).unwrap_or(""));
				if let Err(Error) = self.ApplicationHandle.emit("sky://notification/show", &Parameter) {
					dev_log!("grpc", "warn: [MountainVinegRPCService] Failed to emit sky://notification/show: {}", Error);
				}
			},
			// Cocoon → Mountain: command registration from extensions
			"registerCommand" => {
				let CommandId = Parameter.get("commandId").and_then(|c| c.as_str()).unwrap_or("");
				dev_log!("grpc", "[MountainVinegRPCService] Cocoon registered command: {}", CommandId);
				// Store in CommandRegistry as proxied command → Cocoon handles execution
				if !CommandId.is_empty() {
					if let Ok(mut Registry) = self.RunTime.Environment.ApplicationState.Extension.Registry.CommandRegistry.lock() {
						use crate::Environment::CommandProvider::CommandHandler;
						Registry.insert(
							CommandId.to_string(),
							CommandHandler::Proxied {
								SideCarIdentifier:"cocoon-main".to_string(),
								CommandIdentifier:CommandId.to_string(),
							},
						);
					}
				}
			},
			// Cocoon → Mountain: provider registration from extensions
			"register_hover_provider" | "register_completion_item_provider" |
			"register_definition_provider" | "register_reference_provider" |
			"register_code_actions_provider" | "register_document_symbol_provider" |
			"register_document_formatting_provider" => {
				let Handle = Parameter.get("handle").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
				let Selector = Parameter.get("language_selector").and_then(|s| s.as_str()).unwrap_or("*");
				let ExtId = Parameter.get("extension_id").and_then(|e| e.as_str()).unwrap_or("");
				let ProviderTypeName = MethodName.strip_prefix("register_").and_then(|s| s.strip_suffix("_provider")).unwrap_or("");
				dev_log!("grpc", "[MountainVinegRPCService] Cocoon registered {} provider: handle={}, lang={}", ProviderTypeName, Handle, Selector);
				// Provider registration happens in CocoonService.RegisterProvider via the typed RPC path.
				// This notification path is a fallback for providers registered via the vscode API shim.
				use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType as PT;
				let ProvType = match ProviderTypeName {
					"hover" => Some(PT::Hover),
					"completion_item" => Some(PT::Completion),
					"definition" => Some(PT::Definition),
					"reference" => Some(PT::References),
					"code_actions" => Some(PT::CodeAction),
					"document_symbol" => Some(PT::DocumentSymbol),
					"document_formatting" => Some(PT::DocumentFormatting),
					_ => None,
				};
				if let Some(ProviderType) = ProvType {
					use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;
					let Dto = ProviderRegistrationDTO {
						Handle,
						ProviderType,
						Selector:json!([{ "language": Selector }]),
						SideCarIdentifier:"cocoon-main".to_string(),
						ExtensionIdentifier:json!(ExtId),
						Options:None,
					};
					self.RunTime.Environment.ApplicationState.Extension.ProviderRegistration.RegisterProvider(Handle, Dto);
				}
			},
			_ => {
				dev_log!("grpc", "[MountainVinegRPCService] Cocoon notification: {}", MethodName);
				// Forward all unknown notifications as Tauri events so Wind
				// can subscribe to any Cocoon-originated event.
				let EventName = format!("cocoon:{}", MethodName);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!("grpc", "warn: [MountainVinegRPCService] Failed to emit {}: {}", EventName, Error);
				}
			},
		}

		Ok(Response::new(Empty {}))
	}

	/// Handles a request from Cocoon to cancel a long-running operation.
	///
	/// This method is called when Cocoon wants to cancel an operation that
	/// was previously initiated via process_cocoon_request.
	///
	/// # Parameters
	/// - `request`: CancelOperationRequest with the request ID to cancel
	///
	/// # Returns
	/// - `Ok(Response<Empty>)`: Cancellation was initiated
	/// - `Err(Status)`: Critical error during cancellation
	async fn cancel_operation(&self, request:Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		let cancel_request = request.into_inner();

		let RequestIdentifierToCancel = cancel_request.request_identifier_to_cancel;

		dev_log!("grpc", 
			"[MountainVinegRPCService] Received CancelOperation request for RequestID: {}",
			RequestIdentifierToCancel
		);

		// Look up the operation in the active operations registry
		let cancel_token = {
			let operations = self.ActiveOperations.read().await;
			operations.get(&RequestIdentifierToCancel).cloned()
		};

		match cancel_token {
			Some(token) => {
				// Trigger cancellation token to signal the operation to abort
				token.cancel();

				dev_log!("grpc", 
					"[MountainVinegRPCService] Successfully initiated cancellation for operation {}",
					RequestIdentifierToCancel
				);

				// Note: We don't remove the token here - the operation itself should
				// call UnregisterOperation when it completes. This allows the
				// operation to detect the cancellation and clean up properly.

				Ok(Response::new(Empty {}))
			},
			None => {
				// Operation not found - it may have already completed
				dev_log!("grpc", "warn: [MountainVinegRPCService] Cannot cancel operation {}: operation not found (may have already \
					 completed)",
					RequestIdentifierToCancel);

				// Return success anyway - the operation is not running
				Ok(Response::new(Empty {}))
			},
		}
	}
}
