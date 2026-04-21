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
	dev_log,
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
		dev_log!(
			"grpc",
			"[MountainVinegRPCService] Registered operation {} for cancellation",
			request_id
		);
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
		let ReceiveInstant = std::time::Instant::now();

		dev_log!(
			"grpc",
			"[MountainVinegRPCService] Received gRPC Request [ID: {}]: Method='{}'",
			RequestIdentifier,
			MethodName
		);

		// Hot-path instrumentation (BATCH-16). Every RPC that shows up with
		// uniform 700 ms latency (tree.register, Configuration.Inspect,
		// Command.Execute) emits a `[LandFix:RPC]` marker here so p50/p95 can
		// be derived from the log without patching every handler. The
		// monotonic `t_ns` is a `SystemTime::UNIX_EPOCH` offset so Cocoon's
		// `process.hrtime.bigint()` wire-send stamp can be diffed into three
		// hops: wire → grpc-recv (transit), grpc-recv → dispatch-enter
		// (Track resolve), dispatch-enter → registered (handler body).
		let IsHotRpc = matches!(
			MethodName.as_str(),
			"$tree:register" | "tree.register" | "Configuration.Inspect" | "Command.Execute"
		);
		if IsHotRpc {
			let InstrumentRecvNs = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|D| D.as_nanos())
				.unwrap_or(0);
			dev_log!(
				"grpc",
				"[LandFix:RPC] grpc-recv method={} id={} size={} t_ns={}",
				MethodName,
				RequestIdentifier,
				RequestData.parameter.len(),
				InstrumentRecvNs
			);
		}

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
				// The previous `{:?}` Debug format serialised the full
				// `Value` on every request — cheap for small payloads
				// (`Diagnostic.Clear`), catastrophic for `tree.register` and
				// `Configuration.Inspect` whose options blobs walk recursive
				// structures. Only log param size at the default dev-log
				// level and let the DevLog `all` target surface the body if
				// the caller opts in.
				dev_log!(
					"grpc",
					"[MountainVinegRPCService] Params for [ID: {}] ({} bytes)",
					RequestIdentifier,
					RequestData.parameter.len()
				);
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

		dev_log!(
			"grpc",
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
				if IsHotRpc {
					dev_log!(
						"grpc",
						"[LandFix:RPC] dispatched method={} id={} elapsed={}ms",
						MethodName,
						RequestIdentifier,
						ReceiveInstant.elapsed().as_millis()
					);
				}
				dev_log!(
					"grpc",
					"[MountainVinegRPCService] Request [ID: {}] completed successfully",
					RequestIdentifier
				);

				Ok(Response::new(Self::CreateSuccessResponse(RequestIdentifier, &SuccessfulResult)))
			},

			Err(ErrorString) => {
				// `FileSystem.ReadFile` "Resource not found" is a routine
				// occurrence — extensions probe for optional cache files on
				// activate (terminal-suggest, json-language-features schema
				// associations, etc.). Downgrade 404s to an info-level note
				// so the error log reflects genuine failures only. The
				// response itself is still returned with code -32000 so
				// Cocoon's `readFile` shim can convert it into a proper
				// `vscode.FileSystemError.FileNotFound`.
				let LooksLike404 = MethodName == "FileSystem.ReadFile"
					&& (ErrorString.to_lowercase().contains("resource not found")
						|| ErrorString.to_lowercase().contains("not found"));
				if LooksLike404 {
					dev_log!(
						"grpc",
						"[LandFix:MountainVinegRPC] Request [ID: {}] {} 404 (benign): {}",
						RequestIdentifier,
						MethodName,
						ErrorString
					);
				} else {
					dev_log!(
						"grpc",
						"error: [MountainVinegRPCService] Request [ID: {}] failed: {}",
						RequestIdentifier,
						ErrorString
					);
				}

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

		dev_log!(
			"grpc",
			"[MountainVinegRPCService] Received gRPC Notification: Method='{}'",
			MethodName
		);

		// Validate notification method name
		if MethodName.is_empty() {
			dev_log!(
				"grpc",
				"warn: [MountainVinegRPCService] Received notification with empty method name"
			);
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
				dev_log!(
					"grpc",
					"[MountainVinegRPCService] Extension host message from Cocoon, forwarding to Wind"
				);
				if let Err(Error) = self.ApplicationHandle.emit("cocoon:extensionHostReply", &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] Failed to emit cocoon:extensionHostReply: {}",
						Error
					);
				}
			},
			"ExtensionActivated" => {
				dev_log!("grpc", "[MountainVinegRPCService] Extension activated notification received");
				if let Err(Error) = self.ApplicationHandle.emit("cocoon:extensionActivated", &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] Failed to emit cocoon:extensionActivated: {}",
						Error
					);
				}
			},
			"ExtensionDeactivated" => {
				dev_log!("grpc", "[MountainVinegRPCService] Extension deactivated notification received");
			},
			"WebviewReady" => {
				dev_log!("grpc", "[MountainVinegRPCService] Webview ready notification received");
			},
			// Cocoon → Mountain → Sky: progress notifications emitted by
			// `vscode.window.withProgress`. Each extension progress task fires
			// `progress.start` / `progress.report` / `progress.end` with a
			// unique handle. Mountain normalises them onto the existing
			// `sky://notification/progress-*` channels so Sky's progress
			// indicator renders identically whether the trigger came from an
			// extension or a Mountain handler.
			"progress.start" => {
				let Handle = Parameter.get("handle").and_then(|h| h.as_str()).unwrap_or("");
				let Title = Parameter.get("title").and_then(|h| h.as_str()).unwrap_or("");
				let Cancellable = Parameter.get("cancellable").and_then(|h| h.as_bool()).unwrap_or(false);
				if let Err(Error) = self.ApplicationHandle.emit(
					"sky://notification/progress-begin",
					json!({
						"id": Handle,
						"title": Title,
						"cancellable": Cancellable,
					}),
				) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] sky://notification/progress-begin emit failed: {}",
						Error
					);
				}
			},
			"progress.report" => {
				let Handle = Parameter.get("handle").and_then(|h| h.as_str()).unwrap_or("");
				let Message = Parameter.get("message").and_then(|h| h.as_str()).unwrap_or("");
				let Increment = Parameter.get("increment").and_then(|h| h.as_f64()).unwrap_or(0.0);
				if let Err(Error) = self.ApplicationHandle.emit(
					"sky://notification/progress-update",
					json!({
						"id": Handle,
						"message": Message,
						"increment": Increment,
					}),
				) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] sky://notification/progress-update emit failed: {}",
						Error
					);
				}
			},
			"progress.end" => {
				let Handle = Parameter.get("handle").and_then(|h| h.as_str()).unwrap_or("");
				if let Err(Error) = self
					.ApplicationHandle
					.emit("sky://notification/progress-end", json!({ "id": Handle }))
				{
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] sky://notification/progress-end emit failed: {}",
						Error
					);
				}
			},

			// Cocoon → Mountain → Sky: `vscode.languages.setTextDocumentLanguage(document, languageId)`
			// fires this so Monaco swaps the language mode on the editor.
			"languages.setDocumentLanguage" => {
				if let Err(Error) =
					self.ApplicationHandle.emit("sky://languages/setDocumentLanguage", &Parameter)
				{
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] sky://languages/setDocumentLanguage emit failed: {}",
						Error
					);
				}
			},

			// Cocoon → Mountain → Sky: webview lifecycle notifications from
			// extensions. `webview.setTitle`, `webview.setIconPath`, and the
			// pane-visibility transitions all fan through here.
			"webview.setTitle" | "webview.setIconPath" | "webview.setHtml" => {
				let EventName =
					format!("sky://webview/{}", &MethodName["webview.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] {} emit failed: {}",
						EventName,
						Error
					);
				}
			},

			// Cocoon → Mountain: `vscode.window.createTerminal(...)` is a
			// fire-and-forget from Cocoon's shim. Spawn the PTY via the
			// TerminalProvider so the xterm panel can start receiving data
			// immediately. Emit `sky://terminal/create` with the Cocoon-
			// generated handle so Sky can correlate the panel with the
			// extension-owned terminal instance.
			"window.createTerminal" => {
				use CommonLibrary::{
					Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider,
				};
				let Provider:Arc<dyn TerminalProvider> = self.RunTime.Environment.Require();
				let Name = Parameter
					.get("name")
					.and_then(|V| V.as_str())
					.unwrap_or("terminal")
					.to_string();
				let Options = Parameter.get("options").cloned().unwrap_or_default();
				let Handle = Parameter
					.get("handle")
					.and_then(|V| V.as_str())
					.map(str::to_string)
					.unwrap_or_default();
				let AppHandleForTask = self.ApplicationHandle.clone();
				let NameForTask = Name.clone();
				tokio::spawn(async move {
					let OptionsPayload = if Options.is_object() {
						let mut Map = Options.as_object().cloned().unwrap_or_default();
						Map.entry("name".to_string())
							.or_insert_with(|| json!(NameForTask));
						serde_json::Value::Object(Map)
					} else {
						json!({ "name": NameForTask })
					};
					if let Ok(Created) = Provider.CreateTerminal(OptionsPayload).await {
						if let Err(Error) = AppHandleForTask.emit(
							"sky://terminal/create",
							json!({
								"handle": Handle,
								"id": Created.get("id").cloned().unwrap_or(Value::Null),
								"pid": Created.get("pid").cloned().unwrap_or(Value::Null),
								"name": Created.get("name").cloned().unwrap_or(Value::Null),
							}),
						) {
							dev_log!(
								"grpc",
								"warn: [window.createTerminal] sky://terminal/create emit failed: {}",
								Error
							);
						}
					}
				});
			},

			// Cocoon → Mountain: extension-driven terminal lifecycle. The
			// Cocoon shim for `vscode.window.Terminal` fires these as
			// notifications (fire-and-forget). Route them to Sky so the
			// xterm panel can show/hide the focused terminal; the actual
			// PTY is driven by the same provider the Wind `terminal:*`
			// commands use, so data is already flowing.
			"terminal.sendText" | "terminal.show" | "terminal.hide" | "terminal.dispose" => {
				let EventName = format!("sky://terminal/{}", &MethodName["terminal.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] {} emit failed: {}",
						EventName,
						Error
					);
				}
				// Also drive the provider directly so the underlying PTY
				// responds (sendText) or disposes (dispose). Terminal
				// handles from Cocoon come in the `terminal:N` shape; strip
				// the prefix to recover the numeric identifier.
				let HandleNumeric = Parameter
					.get("handle")
					.and_then(|H| H.as_str())
					.and_then(|S| S.trim_start_matches("terminal:").parse::<u64>().ok());
				if let Some(TerminalId) = HandleNumeric {
					use CommonLibrary::{
						Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider,
					};
					let Provider:Arc<dyn TerminalProvider> = self.RunTime.Environment.Require();
					match MethodName.as_str() {
						"terminal.sendText" => {
							let Text = Parameter
								.get("text")
								.and_then(|T| T.as_str())
								.unwrap_or("")
								.to_string();
							let ProviderForTask = Provider.clone();
							tokio::spawn(async move {
								let _ = ProviderForTask.SendTextToTerminal(TerminalId, Text).await;
							});
						},
						"terminal.dispose" => {
							let ProviderForTask = Provider.clone();
							tokio::spawn(async move {
								let _ = ProviderForTask.DisposeTerminal(TerminalId).await;
							});
						},
						_ => {},
					}
				}
			},

			// Cocoon → Mountain → Sky: `vscode.workspace.applyEdit(edit)`
			// fires this when an extension wants to apply a multi-file
			// WorkspaceEdit. The payload shape matches VS Code's `IWorkspaceEdit`
			// — Sky delegates to its BulkEditService to apply the edits
			// against the open models.
			"workspace.applyEdit" => {
				if let Err(Error) = self.ApplicationHandle.emit("sky://workspace/applyEdit", &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] sky://workspace/applyEdit emit failed: {}",
						Error
					);
				}
			},

			// Cocoon → Mountain → Sky: `vscode.window.showTextDocument(uri, options)`
			// asks the workbench to open and focus a file. Extension activation
			// commonly uses this for "jump to definition" and "reveal config".
			"window.showTextDocument" => {
				if let Err(Error) = self.ApplicationHandle.emit("sky://window/showTextDocument", &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] sky://window/showTextDocument emit failed: {}",
						Error
					);
				}
			},

			// Cocoon → Mountain → Sky: decoration-type lifecycle. Extensions
			// create a decoration type (colour, gutter icon), then apply
			// ranges to it per-editor. Mountain keeps the full lifecycle on
			// the sky:// channel so the editor renderer doesn't need its own
			// gRPC stub.
			"window.createTextEditorDecorationType" | "window.disposeTextEditorDecorationType" => {
				let EventName =
					format!("sky://decoration/{}", &MethodName["window.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] {} emit failed: {}",
						EventName,
						Error
					);
				}
			},

			// Cocoon → Mountain → Sky: debug breakpoint + console
			// notifications. `vscode.debug.addBreakpoints(...)` /
			// `removeBreakpoints(...)` / `onDidReceiveDebugSessionCustomEvent`
			// all fan through here.
			"debug.addBreakpoints" | "debug.removeBreakpoints" | "debug.consoleAppend" => {
				let EventName = format!("sky://debug/{}", &MethodName["debug.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] {} emit failed: {}",
						EventName,
						Error
					);
				}
			},

			// Cocoon → Mountain → Sky: extension-owned output-channel
			// lifecycle. Each `vscode.window.createOutputChannel(...)` fires
			// `outputChannel.create` with a handle + name; subsequent
			// `append`/`clear`/`show`/`hide`/`dispose` reference the handle.
			// Fan every arm to a matching `sky://output-channel/*` event so
			// the workbench's Output panel can render the stream.
			"outputChannel.create"
			| "outputChannel.append"
			| "outputChannel.clear"
			| "outputChannel.show"
			| "outputChannel.hide"
			| "outputChannel.dispose" => {
				let EventName =
					format!("sky://output-channel/{}", &MethodName["outputChannel.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] {} emit failed: {}",
						EventName,
						Error
					);
				}
			},

			// Cocoon → Mountain → Sky: per-item status-bar updates. Each
			// `vscode.window.createStatusBarItem(...)` instance fires
			// `statusBar.update` with its text/tooltip/alignment. Sky's
			// workbench status-bar renderer subscribes to the downstream
			// Tauri event.
			"statusBar.update" | "statusBar.dispose" => {
				let EventName = format!("sky://status-bar/{}", &MethodName["statusBar.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] {} emit failed: {}",
						EventName,
						Error
					);
				}
			},
			// Cocoon → Mountain → Sky: status-bar messages from extensions.
			// Fire as a `sky://status-bar/message` event so the workbench's
			// status-bar component can render the text.
			"statusBar.message" => {
				let Text = Parameter.get("text").and_then(|h| h.as_str()).unwrap_or("");
				let HideAfter = Parameter.get("hideAfter").and_then(|h| h.as_u64());
				if let Err(Error) = self.ApplicationHandle.emit(
					"sky://status-bar/message",
					json!({
						"text": Text,
						"hideAfter": HideAfter,
					}),
				) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] sky://status-bar/message emit failed: {}",
						Error
					);
				}
			},

			// Cocoon → Mountain → Sky: window messages (info/warn/error)
			"window.showMessage" => {
				dev_log!(
					"grpc",
					"[MountainVinegRPCService] Window message from Cocoon: {:?}",
					Parameter.get("message").and_then(|m| m.as_str()).unwrap_or("")
				);
				if let Err(Error) = self.ApplicationHandle.emit("sky://notification/show", &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] Failed to emit sky://notification/show: {}",
						Error
					);
				}
			},
			// Cocoon → Mountain: command registration from extensions
			"registerCommand" => {
				let CommandId = Parameter.get("commandId").and_then(|c| c.as_str()).unwrap_or("");
				dev_log!("grpc", "[MountainVinegRPCService] Cocoon registered command: {}", CommandId);
				// Store in CommandRegistry as proxied command → Cocoon handles execution
				if !CommandId.is_empty() {
					if let Ok(mut Registry) = self
						.RunTime
						.Environment
						.ApplicationState
						.Extension
						.Registry
						.CommandRegistry
						.lock()
					{
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
			// Cocoon → Mountain: unregister a previously-registered command.
			// Paired with `registerCommand` above; removes the proxied
			// CommandHandler so subsequent `commands.executeCommand` no
			// longer routes back to the extension.
			"unregisterCommand" => {
				let CommandId = Parameter.get("commandId").and_then(|c| c.as_str()).unwrap_or("");
				if !CommandId.is_empty() {
					if let Ok(mut Registry) = self
						.RunTime
						.Environment
						.ApplicationState
						.Extension
						.Registry
						.CommandRegistry
						.lock()
					{
						Registry.remove(CommandId);
						dev_log!("grpc", "[MountainVinegRPCService] Cocoon unregistered command: {}", CommandId);
					}
				}
			},

			// Cocoon → Mountain: provider registration from extensions
			"register_hover_provider"
			| "register_completion_item_provider"
			| "register_definition_provider"
			| "register_reference_provider"
			| "register_code_actions_provider"
			| "register_document_symbol_provider"
			| "register_document_formatting_provider" => {
				let Handle = Parameter.get("handle").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
				let Selector = Parameter.get("language_selector").and_then(|s| s.as_str()).unwrap_or("*");
				let ExtId = Parameter.get("extension_id").and_then(|e| e.as_str()).unwrap_or("");
				let ProviderTypeName = MethodName
					.strip_prefix("register_")
					.and_then(|s| s.strip_suffix("_provider"))
					.unwrap_or("");
				dev_log!(
					"grpc",
					"[MountainVinegRPCService] Cocoon registered {} provider: handle={}, lang={}",
					ProviderTypeName,
					Handle,
					Selector
				);
				// Provider registration happens in CocoonService.RegisterProvider via the typed
				// RPC path. This notification path is a fallback for providers registered
				// via the vscode API shim.
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
					self.RunTime
						.Environment
						.ApplicationState
						.Extension
						.ProviderRegistration
						.RegisterProvider(Handle, Dto);
				}
			},
			_ => {
				dev_log!("grpc", "[MountainVinegRPCService] Cocoon notification: {}", MethodName);
				// Forward all unknown notifications as Tauri events so Wind
				// can subscribe to any Cocoon-originated event.
				let EventName = format!("cocoon:{}", MethodName);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] Failed to emit {}: {}",
						EventName,
						Error
					);
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

		dev_log!(
			"grpc",
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

				dev_log!(
					"grpc",
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
				dev_log!(
					"grpc",
					"warn: [MountainVinegRPCService] Cannot cancel operation {}: operation not found (may have \
					 already completed)",
					RequestIdentifierToCancel
				);

				// Return success anyway - the operation is not running
				Ok(Response::new(Empty {}))
			},
		}
	}
}
