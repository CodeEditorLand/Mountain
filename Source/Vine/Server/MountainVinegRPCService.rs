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
	/// Accessor for Tauri `AppHandle` - used by the per-wire-method atoms
	/// in `Vine::Server::Notification::*` that need to emit
	/// `sky://` / `cocoon:*` events downstream. Kept as a thin read so the
	/// struct's fields can stay private; atoms should never mutate the
	/// handle, only `emit` through it.
	pub fn ApplicationHandle(&self) -> &AppHandle { &self.ApplicationHandle }

	/// Accessor for the shared `ApplicationRunTime`. Notification atoms
	/// reach `Environment.ApplicationState.*` (provider registry, extension
	/// registry, scheduler) through this. Clone from `Arc` when the atom
	/// needs to keep it across an `.await` boundary.
	pub fn RunTime(&self) -> &Arc<ApplicationRunTime> { &self.RunTime }
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
				// `Value` on every request - cheap for small payloads
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
				// occurrence - extensions probe for optional cache files on
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
				if let Err(Error) = self.ApplicationHandle.emit("sky://languages/setDocumentLanguage", &Parameter) {
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
				let EventName = format!("sky://webview/{}", &MethodName["webview.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
				}
			},

			// Cocoon → Mountain: `vscode.window.createTerminal(...)` is a
			// fire-and-forget from Cocoon's shim. Spawn the PTY via the
			// TerminalProvider so the xterm panel can start receiving data
			// immediately. Emit `sky://terminal/create` with the Cocoon-
			// generated handle so Sky can correlate the panel with the
			// extension-owned terminal instance.
			"window.createTerminal" => {
				use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};
				let Provider:Arc<dyn TerminalProvider> = self.RunTime.Environment.Require();
				let Name = Parameter.get("name").and_then(|V| V.as_str()).unwrap_or("terminal").to_string();
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
						Map.entry("name".to_string()).or_insert_with(|| json!(NameForTask));
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
					dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
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
						Environment::Requires::Requires,
						Terminal::TerminalProvider::TerminalProvider,
					};
					let Provider:Arc<dyn TerminalProvider> = self.RunTime.Environment.Require();
					match MethodName.as_str() {
						"terminal.sendText" => {
							let Text = Parameter.get("text").and_then(|T| T.as_str()).unwrap_or("").to_string();
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
			// - Sky delegates to its BulkEditService to apply the edits
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
				let EventName = format!("sky://decoration/{}", &MethodName["window.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
				}
			},

			// Cocoon → Mountain → Sky: debug breakpoint + console
			// notifications. `vscode.debug.addBreakpoints(...)` /
			// `removeBreakpoints(...)` / `onDidReceiveDebugSessionCustomEvent`
			// all fan through here.
			"debug.addBreakpoints" | "debug.removeBreakpoints" | "debug.consoleAppend" => {
				let EventName = format!("sky://debug/{}", &MethodName["debug.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
				}
			},

			// NOTE: `outputChannel.*` variants are dispatched from the
			// Batch 9 block further down this match (see
			// `Vine::Server::Notification::OutputChannel*`). Previously a
			// legacy OR-pattern lived here fanning to
			// `sky://output-channel/*` (hyphenated) which no Sky listener
			// subscribed to - every output-channel write silently dropped.
			// Intentionally no arm here so the Batch 9 atoms win; do not
			// re-add without removing the atom dispatch.

			// Cocoon → Mountain → Sky: per-item status-bar updates. Each
			// `vscode.window.createStatusBarItem(...)` instance fires
			// `statusBar.update` with its text/tooltip/alignment. Sky's
			// workbench status-bar renderer subscribes to the downstream
			// Tauri event. Canonical channel prefix is `sky://statusbar/`
			// (no hyphen) to match the `sky://statusbar/*` family every
			// other emit site uses.
			"statusBar.update" | "statusBar.dispose" => {
				let EventName = format!("sky://statusbar/{}", &MethodName["statusBar.".len()..]);
				if let Err(Error) = self.ApplicationHandle.emit(&EventName, &Parameter) {
					dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
				}
			},
			// Cocoon → Mountain → Sky: status-bar messages from extensions.
			// Canonical channel is `sky://statusbar/set-message` (matching
			// the rest of the `sky://statusbar/*` family); the legacy
			// `sky://status-bar/message` fork has been retired.
			"statusBar.message" => {
				let Text = Parameter.get("text").and_then(|h| h.as_str()).unwrap_or("");
				let HideAfter = Parameter.get("hideAfter").and_then(|h| h.as_u64());
				if let Err(Error) = self.ApplicationHandle.emit(
					"sky://statusbar/set-message",
					json!({
						"text": Text,
						"hideAfter": HideAfter,
					}),
				) {
					dev_log!(
						"grpc",
						"warn: [MountainVinegRPCService] sky://statusbar/set-message emit failed: {}",
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

			// Batch 8: provider unregister atoms. Each wire method lives in
			// its own `Notification/<Name>.rs` atom - the arm is a pure
			// delegation so adding a variant stays a one-line change here
			// plus one new file.
			"unregister_authentication_provider" => {
				super::Notification::UnregisterAuthenticationProvider::UnregisterAuthenticationProvider(self, &Parameter).await;
			},
			"unregister_debug_adapter" => {
				super::Notification::UnregisterDebugAdapter::UnregisterDebugAdapter(self, &Parameter).await;
			},
			"unregister_file_system_provider" => {
				super::Notification::UnregisterFileSystemProvider::UnregisterFileSystemProvider(self, &Parameter).await;
			},
			"unregister_scm_provider" => {
				super::Notification::UnregisterScmProvider::UnregisterScmProvider(self, &Parameter).await;
			},
			"unregister_task_provider" => {
				super::Notification::UnregisterTaskProvider::UnregisterTaskProvider(self, &Parameter).await;
			},
			"unregister_uri_handler" => {
				super::Notification::UnregisterUriHandler::UnregisterUriHandler(self, &Parameter).await;
			},
			"update_scm_group" => {
				super::Notification::UpdateScmGroup::UpdateScmGroup(self, &Parameter).await;
			},

			// Batch 11: progress lifecycle name alignment.
			"progress.update" => {
				super::Notification::ProgressUpdate::ProgressUpdate(self, &Parameter).await;
			},
			"progress.complete" => {
				super::Notification::ProgressComplete::ProgressComplete(self, &Parameter).await;
			},

			// Batch 10: status-bar text-only fast path + item disposal.
			"setStatusBarText" => {
				super::Notification::SetStatusBarText::SetStatusBarText(self, &Parameter).await;
			},
			"disposeStatusBarItem" => {
				super::Notification::DisposeStatusBarItem::DisposeStatusBarItem(self, &Parameter).await;
			},

			// Batch 9: output channel lifecycle. Two parallel wire names
			// (`output.*` via `MountainClient.sendNotification` and
			// `outputChannel.*` via `SendToMountain`) both forward to the
			// same `sky://output/*` channels until Cocoon consolidates.
			"output.create" => {
				super::Notification::OutputCreate::OutputCreate(self, &Parameter).await;
			},
			"output.append" => {
				super::Notification::OutputAppend::OutputAppend(self, &Parameter).await;
			},
			"output.appendLine" => {
				super::Notification::OutputAppendLine::OutputAppendLine(self, &Parameter).await;
			},
			"output.clear" => {
				super::Notification::OutputClear::OutputClear(self, &Parameter).await;
			},
			"output.show" => {
				super::Notification::OutputShow::OutputShow(self, &Parameter).await;
			},
			"output.dispose" => {
				super::Notification::OutputDispose::OutputDispose(self, &Parameter).await;
			},
			"output.replace" => {
				super::Notification::OutputReplace::OutputReplace(self, &Parameter).await;
			},
			"outputChannel.create" => {
				super::Notification::OutputChannelCreate::OutputChannelCreate(self, &Parameter).await;
			},
			"outputChannel.append" => {
				super::Notification::OutputChannelAppend::OutputChannelAppend(self, &Parameter).await;
			},
			"outputChannel.clear" => {
				super::Notification::OutputChannelClear::OutputChannelClear(self, &Parameter).await;
			},
			"outputChannel.show" => {
				super::Notification::OutputChannelShow::OutputChannelShow(self, &Parameter).await;
			},
			"outputChannel.hide" => {
				super::Notification::OutputChannelHide::OutputChannelHide(self, &Parameter).await;
			},
			"outputChannel.dispose" => {
				super::Notification::OutputChannelDispose::OutputChannelDispose(self, &Parameter).await;
			},

			// Batch 13: webview reverse-channel (Mountain → renderer).
			"webview.postMessage" => {
				super::Notification::WebviewPostMessage::WebviewPostMessage(self, &Parameter).await;
			},
			"webview.dispose" => {
				super::Notification::WebviewDispose::WebviewDispose(self, &Parameter).await;
			},

			// Batch 14: grammar config, external-URI open, security alert.
			"set_language_configuration" => {
				super::Notification::SetLanguageConfiguration::SetLanguageConfiguration(self, &Parameter).await;
			},
			"openExternal" => {
				super::Notification::OpenExternal::OpenExternal(self, &Parameter).await;
			},
			"security.incident" => {
				super::Notification::SecurityIncident::SecurityIncident(self, &Parameter).await;
			},

			// Cocoon → Mountain: provider registration from extensions.
			//
			// Covers all 34 `register_*` / `register_*_provider` notification
			// variants that Cocoon's vscode-API shim emits. Each lands in
			// Mountain's `ProviderRegistration` keyed on `Handle`; the
			// language-feature RPC path (e.g. GetHoverAtPosition) then looks
			// up the handle and proxies back to Cocoon with the original
			// `$providerXxx` method.
			//
			// Wire-method naming: the shim uses snake_case with two trailing
			// shapes - plain verbs (`register_rename`) and `_provider` suffix
			// (`register_hover_provider`). The map below strips both.
			// Full list mirrors Cocoon's `vscode` API shim wire strings - the
			// authoritative set grep'd from `Cocoon/Source` is: most providers
			// carry a `_provider` suffix; a handful (debug_adapter,
			// uri_handler, external_uri_opener, notebook_serializer,
			// remote_authority_resolver, resource_label_formatter,
			// scm_resource_group) do not. Keep both the suffixed and
			// non-suffixed variants listed explicitly so the OR-match stays
			// readable at a glance; the strip-logic below normalises either
			// form into `ProviderTypeName` for the enum lookup.
			"register_authentication_provider"
			| "register_call_hierarchy_provider"
			| "register_code_actions_provider"
			| "register_code_lens_provider"
			| "register_color_provider"
			| "register_completion_item_provider"
			| "register_debug_adapter"
			| "register_debug_configuration_provider"
			| "register_declaration_provider"
			| "register_definition_provider"
			| "register_document_drop_edit_provider"
			| "register_document_formatting_provider"
			| "register_document_highlight_provider"
			| "register_document_link_provider"
			| "register_document_paste_edit_provider"
			| "register_document_range_formatting_provider"
			| "register_document_symbol_provider"
			| "register_evaluatable_expression_provider"
			| "register_external_uri_opener"
			| "register_file_decoration_provider"
			| "register_file_system_provider"
			| "register_folding_range_provider"
			| "register_hover_provider"
			| "register_implementation_provider"
			| "register_inlay_hints_provider"
			| "register_inline_completion_item_provider"
			| "register_inline_edit_provider"
			| "register_inline_values_provider"
			| "register_linked_editing_range_provider"
			| "register_mapped_edits_provider"
			| "register_multi_document_highlight_provider"
			| "register_notebook_content_provider"
			| "register_notebook_serializer"
			| "register_on_type_formatting_provider"
			| "register_reference_provider"
			| "register_remote_authority_resolver"
			| "register_rename_provider"
			| "register_resource_label_formatter"
			| "register_scm_provider"
			| "register_scm_resource_group"
			| "register_selection_range_provider"
			| "register_semantic_tokens_provider"
			| "register_signature_help_provider"
			| "register_task_provider"
			| "register_terminal_link_provider"
			| "register_terminal_profile_provider"
			| "register_text_document_content_provider"
			| "register_type_definition_provider"
			| "register_type_hierarchy_provider"
			| "register_uri_handler"
			| "register_workspace_symbol_provider" => {
				let Handle = Parameter.get("handle").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
				let Selector = Parameter.get("language_selector").and_then(|s| s.as_str()).unwrap_or("*");
				let ExtId = Parameter.get("extension_id").and_then(|e| e.as_str()).unwrap_or("");
				// Extension-scoped scheme (for FileSystemProvider, TextDocumentContentProvider,
				// UriHandler). Present only for schema-bound variants; `""` for others.
				let Scheme = Parameter.get("scheme").and_then(|s| s.as_str()).unwrap_or("");
				let ProviderTypeName = MethodName
					.strip_prefix("register_")
					.map(|Stripped| Stripped.strip_suffix("_provider").unwrap_or(Stripped))
					.unwrap_or("");
				dev_log!(
					"grpc",
					"[MountainVinegRPCService] Cocoon registered {} provider: handle={}, lang={}",
					ProviderTypeName,
					Handle,
					Selector
				);
				dev_log!(
					"provider-register",
					"[ProviderRegister] accepted method={} type={} handle={} lang={} scheme={} ext={}",
					MethodName,
					ProviderTypeName,
					Handle,
					Selector,
					Scheme,
					ExtId
				);
				use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType as PT;
				let ProvType = match ProviderTypeName {
					"authentication" => Some(PT::Authentication),
					"call_hierarchy" => Some(PT::CallHierarchy),
					"code_actions" => Some(PT::CodeAction),
					"code_lens" => Some(PT::CodeLens),
					"color" => Some(PT::Color),
					"completion_item" => Some(PT::Completion),
					"debug_adapter" => Some(PT::DebugAdapter),
					"debug_configuration" => Some(PT::DebugConfiguration),
					"declaration" => Some(PT::Declaration),
					"definition" => Some(PT::Definition),
					"document_drop_edit" => Some(PT::DocumentDropEdit),
					"document_formatting" => Some(PT::DocumentFormatting),
					"document_highlight" => Some(PT::DocumentHighlight),
					"document_link" => Some(PT::DocumentLink),
					"document_paste_edit" => Some(PT::DocumentPasteEdit),
					"document_range_formatting" => Some(PT::DocumentRangeFormatting),
					"document_symbol" => Some(PT::DocumentSymbol),
					"evaluatable_expression" => Some(PT::EvaluatableExpression),
					"external_uri_opener" => Some(PT::ExternalUriOpener),
					"file_decoration" => Some(PT::FileDecoration),
					"file_system" => Some(PT::FileSystem),
					"folding_range" => Some(PT::FoldingRange),
					"hover" => Some(PT::Hover),
					"implementation" => Some(PT::Implementation),
					"inlay_hints" => Some(PT::InlayHint),
					"inline_completion_item" => Some(PT::InlineCompletion),
					"inline_edit" => Some(PT::InlineEdit),
					"inline_values" => Some(PT::InlineValues),
					"linked_editing_range" => Some(PT::LinkedEditingRange),
					"mapped_edits" => Some(PT::MappedEdits),
					"multi_document_highlight" => Some(PT::MultiDocumentHighlight),
					"notebook_content" => Some(PT::NotebookContent),
					"notebook_serializer" => Some(PT::NotebookSerializer),
					"on_type_formatting" => Some(PT::OnTypeFormatting),
					"reference" => Some(PT::References),
					"remote_authority_resolver" => Some(PT::RemoteAuthorityResolver),
					"rename" => Some(PT::Rename),
					"resource_label_formatter" => Some(PT::ResourceLabelFormatter),
					"scm" => Some(PT::SourceControl),
					"scm_resource_group" => Some(PT::ScmResourceGroup),
					"selection_range" => Some(PT::SelectionRange),
					"semantic_tokens" => Some(PT::SemanticTokens),
					"signature_help" => Some(PT::SignatureHelp),
					"task" => Some(PT::Task),
					"terminal_link" => Some(PT::TerminalLink),
					"terminal_profile" => Some(PT::TerminalProfile),
					"text_document_content" => Some(PT::TextDocumentContent),
					"type_definition" => Some(PT::TypeDefinition),
					"type_hierarchy" => Some(PT::TypeHierarchy),
					"uri_handler" => Some(PT::UriHandler),
					"workspace_symbol" => Some(PT::WorkspaceSymbol),
					_ => None,
				};
				if let Some(ProviderType) = ProvType {
					use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;
					// Scheme-bound providers carry their scheme in the selector payload so
					// the Mountain-side resolver (FileSystem router, URI handler dispatch,
					// TextDocumentContent view, …) can match on it.
					let SelectorValue = if !Scheme.is_empty() {
						json!([{ "scheme": Scheme, "language": Selector }])
					} else {
						json!([{ "language": Selector }])
					};
					let Dto = ProviderRegistrationDTO {
						Handle,
						ProviderType,
						Selector:SelectorValue,
						SideCarIdentifier:"cocoon-main".to_string(),
						ExtensionIdentifier:json!(ExtId),
						Options:Parameter.get("options").cloned(),
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
				// No typed match arm exists for this notification - it hits
				// the default path and becomes a `cocoon:<method>` Tauri
				// event that Wind may or may not listen for. The
				// `notif-drop` tag surfaces every fall-through so we can
				// tell at a glance which notifications Cocoon emits that
				// Mountain has no first-class handler for (register_*
				// provider variants beyond the seven handled above,
				// register_debug_adapter, register_task_provider,
				// register_uri_handler, register_file_system_provider, …).
				dev_log!(
					"notif-drop",
					"[NotifDrop] method={} payload_bytes={} (falls through to cocoon:{} event)",
					MethodName,
					NotificationData.parameter.len(),
					MethodName
				);
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
