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
use ::Vine::Generated::mountain_service_server::MountainService;

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
	// LAND-PATCH B7-S6 P2: bidirectional streaming channel.
	// Stub for now - the multiplexer that drains incoming Envelopes
	// and dispatches to the unary handler tree is implemented in a
	// follow-up patch (Patch 14). Until then this returns
	// `Unimplemented` so callers fall back to the unary path.
	type OpenChannelFromCocoonStream = std::pin::Pin<
		Box<
			dyn tonic::codegen::tokio_stream::Stream<Item = Result<crate::Vine::Generated::Envelope, tonic::Status>>
				+ Send
				+ 'static,
		>,
	>;

	async fn open_channel_from_cocoon(
		&self,

		_request:tonic::Request<tonic::Streaming<crate::Vine::Generated::Envelope>>,
	) -> Result<tonic::Response<Self::OpenChannelFromCocoonStream>, tonic::Status> {
		Err(tonic::Status::unimplemented(
			"OpenChannelFromCocoon: streaming multiplexer not yet wired (Patch 14); use unary endpoints",
		))
	}

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

		// Single consolidated receive line - replaces the previous
		// three-line burst (`Received…` + `Params for…` + `Dispatching…`)
		// that fired per RPC × thousands of RPCs per session. One log
		// statement is enough to reconstruct the request lifecycle from
		// the file: method, id, payload size in bytes. Gated under
		// `grpc-verbose` so the default `short` trace stays quiet;
		// failures still flow through the `grpc` (non-verbose) tag in
		// the validate / dispatch error paths below.
		dev_log!(
			"grpc-verbose",
			"[MountainVinegRPCService] recv id={} method={} size={}B",
			RequestIdentifier,
			MethodName,
			RequestData.parameter.len()
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

			// Per-call receive timestamp for latency diagnosis - only
			// useful when actively profiling. Gate under `rpc-latency`
			// so `short` / `grpc` don't print it.
			dev_log!(
				"rpc-latency",
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

		// Deserialize JSON parameters. The byte-count + method are
		// already captured in the consolidated `recv id=…` line above;
		// no additional `Params for [ID: …]` emit is needed.
		let ParametersValue:Value = match serde_json::from_slice(&RequestData.parameter) {
			Ok(v) => v,

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

		// Dispatch line removed - the `recv id=… method=…` line above
		// is the single source of truth for "this RPC started"; the
		// completion path emits its own line on success / error.

		// Dispatch request to Track module for processing
		let DispatchResult = Track::SideCarRequest::DispatchSideCarRequest::DispatchSideCarRequest(
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
					// Hot-RPC dispatched latency line - already narrow
					// (~50 tagged RPCs per session). Route to `rpc-latency`
					// so the profiling context stays opt-in.
					dev_log!(
						"rpc-latency",
						"[LandFix:RPC] dispatched method={} id={} elapsed={}ms",
						MethodName,
						RequestIdentifier,
						ReceiveInstant.elapsed().as_millis()
					);
				}

				// Success completion fires per request (14k+ in long sessions).
				// Failures still log under the unconditional `error:` path
				// below, so routing this to `grpc-verbose` doesn't hide real
				// problems.
				dev_log!(
					"grpc-verbose",
					"[MountainVinegRPCService] Request [ID: {}] completed successfully",
					RequestIdentifier
				);

				Ok(Response::new(Self::CreateSuccessResponse(RequestIdentifier, &SuccessfulResult)))
			},

			Err(ErrorString) => {
				// Routine 404s - extensions probe for optional workspace
				// files on activate:
				//   - `FileSystem.ReadFile` → missing cache files (terminal-suggest, JSON
				//     schema associations, composer.json, Gemfile.lock, Drupal.php).
				//   - `FileSystem.Stat` → optional config probes.
				// Both surface as "resource not found" / "not found" /
				// "ENOENT". Downgrade to `grpc-verbose` so the default
				// log reflects genuine failures only. The response still
				// returns -32000 so Cocoon's shim can convert it to a
				// proper `vscode.FileSystemError.FileNotFound`.
				let LowerError = ErrorString.to_lowercase();

				// "Path is outside of the registered workspace folders" /
				// "Permission denied" responses come from the path-security
				// guard in `Environment/Utility/PathSecurity.rs` when an
				// extension probes a directory outside the open workspace
				// (Svelte's `enableContextMenu` walks every `package.json`
				// in the entire workspace tree, including out-of-root
				// submodule dependencies). From the extension's perspective
				// these are equivalent to "file not present" and must NOT
				// count against Cocoon's circuit breaker - a workspace with
				// many sibling submodules trips the breaker open within the
				// first few hundred ms of activation otherwise.
				let LooksLike404 = (MethodName == "FileSystem.ReadFile"
					|| MethodName == "FileSystem.Stat"
					|| MethodName == "FileSystem.ReadDirectory")
					&& (LowerError.contains("resource not found")
						|| LowerError.contains("not found")
						|| LowerError.contains("enoent")
						|| LowerError.contains("no such file or directory")
						|| LowerError.contains("entity not found")
						|| LowerError.contains("os error 2")
						|| LowerError.contains("path is outside of the registered workspace")
						|| LowerError.contains("permission denied for operation")
						|| LowerError.contains("workspace is not trusted"));

				if LooksLike404 {
					dev_log!(
						"grpc-verbose",
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

				// Distinct code -32004 for benign 404s lets the Cocoon shim
				// classify them without a string-regex round-trip. -32000
				// stays the catch-all for genuine failures.
				let ErrorCode = if LooksLike404 { -32004 } else { -32000 };

				Ok(Response::new(Self::CreateErrorResponse(
					RequestIdentifier,
					ErrorCode,
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

		// Notifications are even higher-volume than requests
		// (progress.report alone fires 2500+ times per long activation).
		// Move under `grpc-verbose` alongside the request-side banner.
		dev_log!(
			"grpc-verbose",
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
			// Batch 15: extension-host + progress + languages arms now live
			// as atoms under `Vine::Server::Notification::*`. Each match arm
			// is pure delegation - adding a new wire method is a one-line
			// change here plus one new atom file.
			"extensionHostMessage" => {
				::Vine::Server::Notification::ExtensionHostMessage::ExtensionHostMessage(self, &Parameter).await;
			},

			"ExtensionActivated" => {
				::Vine::Server::Notification::ExtensionActivated::ExtensionActivated(self, &Parameter).await;
			},

			"ExtensionDeactivated" => {
				::Vine::Server::Notification::ExtensionDeactivated::ExtensionDeactivated(self, &Parameter).await;
			},

			"WebviewReady" => {
				::Vine::Server::Notification::WebviewReady::WebviewReady(self, &Parameter).await;
			},

			"progress.start" => {
				::Vine::Server::Notification::ProgressStart::ProgressStart(self, &Parameter).await;
			},

			"progress.report" => {
				::Vine::Server::Notification::ProgressReport::ProgressReport(self, &Parameter).await;
			},

			"progress.end" => {
				::Vine::Server::Notification::ProgressEnd::ProgressEnd(self, &Parameter).await;
			},

			"languages.setDocumentLanguage" => {
				::Vine::Server::Notification::LanguagesSetDocumentLanguage::LanguagesSetDocumentLanguage(self, &Parameter).await;
			},

			"workspace.applyEdit" => {
				::Vine::Server::Notification::WorkspaceApplyEdit::WorkspaceApplyEdit(self, &Parameter).await;
			},

			"window.showTextDocument" => {
				::Vine::Server::Notification::WindowShowTextDocument::WindowShowTextDocument(self, &Parameter).await;
			},

			// Batch 16: the remaining Cocoon-notification arms, now pure
			// atom delegations. Each wire method lives in its own file
			// under `Vine::Server::Notification::*`. "Group atoms"
			// (TerminalLifecycle, DebugLifecycle, WebviewLifecycle, etc.)
			// handle 3-4 wire methods that share the same relay pattern.
			"webview.setTitle"
			| "webview.setIconPath"
			| "webview.setHtml"
			| "webview.setOptions"
			| "webview.updateView"
			| "webview.reveal" => {
				::Vine::Server::Notification::WebviewLifecycle::WebviewLifecycle(self, &MethodName, &Parameter).await;
			},

			"window.createTerminal" => {
				::Vine::Server::Notification::WindowCreateTerminal::WindowCreateTerminal(self, &Parameter).await;
			},

			"terminal.sendText" | "terminal.show" | "terminal.hide" | "terminal.dispose" => {
				::Vine::Server::Notification::TerminalLifecycle::TerminalLifecycle(self, &MethodName, &Parameter).await;
			},

			// Tree view refresh - extension fired its `onDidChangeTreeData`
			// event. Relay to Sky which calls `ITreeView.refresh()` to
			// trigger a fresh getChildren() round-trip.
			"tree.refresh" => {
				::Vine::Server::Notification::TreeRefresh::TreeRefresh(self, &Parameter).await;
			},

			// EnvironmentVariableCollection mutations - applied to every
			// PTY spawn that follows. The variant dispatch lives in the
			// notification module since each op writes to the same global
			// registry.
			"terminal.envCollection.replace"
			| "terminal.envCollection.append"
			| "terminal.envCollection.prepend"
			| "terminal.envCollection.delete"
			| "terminal.envCollection.clear"
			| "terminal.envCollection.setPersistent"
			| "terminal.envCollection.setDescription" => {
				super::Notification::TerminalEnvCollection::TerminalEnvCollectionDispatch(
					self,
					&MethodName,
					&Parameter,
				)
				.await;
			},

			"window.createTextEditorDecorationType" | "window.disposeTextEditorDecorationType" => {
				::Vine::Server::Notification::DecorationTypeLifecycle::DecorationTypeLifecycle(self, &MethodName, &Parameter)
					.await;
			},

			// Extension called `editor.setDecorations(type, ranges)`.
			// Batched and emitted as `sky://decoration/set-ranges` so Sky can
			// apply the ranges to the Monaco editor for the matching URI.
			"window.setTextEditorDecorations" => {
				::Vine::Server::Notification::SetTextEditorDecorations::SetTextEditorDecorations(self, &Parameter).await;
			},

			// Extension called `editor.edit(cb)` - an in-place text mutation.
			// Payload: `{ uri, edits: [{range, text}] }`.
			// Sky applies via `ICodeEditorService` → `editor.executeEdits`.
			"window.applyTextEdits" => {
				::Vine::Server::Notification::ApplyTextEdits::ApplyTextEdits(self, &Parameter).await;
			},

			"debug.addBreakpoints" | "debug.removeBreakpoints" | "debug.consoleAppend" => {
				::Vine::Server::Notification::DebugLifecycle::DebugLifecycle(self, &MethodName, &Parameter).await;
			},

			"statusBar.update" | "statusBar.dispose" => {
				::Vine::Server::Notification::StatusBarLifecycle::StatusBarLifecycle(self, &MethodName, &Parameter).await;
			},

			"statusBar.message" => {
				::Vine::Server::Notification::StatusBarMessage::StatusBarMessage(self, &Parameter).await;
			},

			"window.showMessage" => {
				::Vine::Server::Notification::WindowShowMessage::WindowShowMessage(self, &Parameter).await;
			},

			"registerCommand" => {
				::Vine::Server::Notification::RegisterCommand::RegisterCommand(self, &Parameter).await;
			},

			"unregisterCommand" => {
				::Vine::Server::Notification::UnregisterCommand::UnregisterCommand(self, &Parameter).await;
			},

			// NOTE: `outputChannel.*` arms were previously here fanning to
			// the wrong `sky://output-channel/*` channel. Batch 9 atoms
			// below correctly route to `sky://output/*`; the legacy arm
			// was removed to stop it from shadowing the atoms.

			// Batch 8: provider unregister atoms. Each wire method lives in
			// its own `Notification/<Name>.rs` atom - the arm is a pure
			// delegation so adding a variant stays a one-line change here
			// plus one new file.
			// Pure provider-unregistration atoms: read handle, call VineHost,
			// log. No intermediate file needed - call Vine's support helper
			// directly. Atoms with extra logic (scheme log, handle computation,
			// sky relay) go through a named Vine atom.
			"unregister_authentication_provider" => {
				::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(
					self,
					&Parameter,
					"authentication",
				);
			},

			"unregister_debug_adapter" => {
				::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(
					self,
					&Parameter,
					"debug_adapter",
				);
			},

			"unregister_debug_configuration_provider" => {
				::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(
					self,
					&Parameter,
					"debug_configuration",
				);
			},

			"unregister_external_uri_opener" => {
				::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(
					self,
					&Parameter,
					"external_uri_opener",
				);
			},

			"unregister_remote_authority_resolver" => {
				::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(
					self,
					&Parameter,
					"remote_authority_resolver",
				);
			},

			"unregister_task_provider" => {
				::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(self, &Parameter, "task");
			},

			// These three have extra logic in their Vine atoms (scheme log,
			// scmId handle computation + sky relay, URI scheme log).
			"unregister_file_system_provider" => {
				::Vine::Server::Notification::UnregisterFileSystemProvider::UnregisterFileSystemProvider(
					self, &Parameter,
				)
				.await;
			},

			"unregister_scm_provider" => {
				::Vine::Server::Notification::UnregisterScmProvider::UnregisterScmProvider(self, &Parameter).await;
			},

			"unregister_uri_handler" => {
				::Vine::Server::Notification::UnregisterUriHandler::UnregisterUriHandler(self, &Parameter).await;
			},

			"update_scm_group" => {
				::Vine::Server::Notification::UpdateScmGroup::UpdateScmGroup(self, &Parameter).await;
			},

			// SCM register pair: explicit arms BEFORE the language-providers
			// OR-block below. Without these, both `register_scm_provider` and
			// `register_scm_resource_group` fell into the catch-all language-
			// providers branch which only writes to
			// `Extension::ProviderRegistration` - never to
			// `ApplicationState::Feature::Markers::SourceControlManagement*`,
			// so the SCM viewlet stayed empty even after vscode.git's
			// `createSourceControl(...)` round-tripped successfully. The new
			// atoms write the markers + emit the `sky://scm/*` events the
			// renderer subscribes to.
			"register_scm_provider" => {
				::Vine::Server::Notification::RegisterScmProvider::RegisterScmProvider(self, &Parameter).await;
			},

			"register_scm_resource_group" => {
				::Vine::Server::Notification::RegisterScmResourceGroup::RegisterScmResourceGroup(self, &Parameter).await;
			},

			// Batch 11: progress lifecycle name alignment.
			"progress.update" => {
				::Vine::Server::Notification::ProgressUpdate::ProgressUpdate(self, &Parameter).await;
			},

			"progress.complete" => {
				::Vine::Server::Notification::ProgressComplete::ProgressComplete(self, &Parameter).await;
			},

			// Batch 10: status-bar text-only fast path + item disposal.
			"setStatusBarText" => {
				::Vine::Server::Notification::SetStatusBarText::SetStatusBarText(self, &Parameter).await;
			},

			"disposeStatusBarItem" => {
				::Vine::Server::Notification::DisposeStatusBarItem::DisposeStatusBarItem(self, &Parameter).await;
			},

			// Batch 9: output channel lifecycle. Two parallel wire names
			// (`output.*` via `MountainClient.sendNotification` and
			// `outputChannel.*` via `SendToMountain`) both forward to the
			// same `sky://output/*` channels until Cocoon consolidates.
			"output.create" => {
				::Vine::Server::Notification::OutputCreate::OutputCreate(self, &Parameter).await;
			},

			"output.append" => {
				::Vine::Server::Notification::OutputAppend::OutputAppend(self, &Parameter).await;
			},

			"output.appendLine" => {
				::Vine::Server::Notification::OutputAppendLine::OutputAppendLine(self, &Parameter).await;
			},

			"output.clear" => {
				::Vine::Server::Notification::OutputClear::OutputClear(self, &Parameter).await;
			},

			"output.show" => {
				::Vine::Server::Notification::OutputShow::OutputShow(self, &Parameter).await;
			},

			"output.dispose" => {
				::Vine::Server::Notification::OutputDispose::OutputDispose(self, &Parameter).await;
			},

			"output.replace" => {
				::Vine::Server::Notification::OutputReplace::OutputReplace(self, &Parameter).await;
			},

			"outputChannel.create" => {
				::Vine::Server::Notification::OutputChannelCreate::OutputChannelCreate(self, &Parameter).await;
			},

			"outputChannel.append" => {
				::Vine::Server::Notification::OutputChannelAppend::OutputChannelAppend(self, &Parameter).await;
			},

			"outputChannel.clear" => {
				::Vine::Server::Notification::OutputChannelClear::OutputChannelClear(self, &Parameter).await;
			},

			"outputChannel.replace" => {
				::Vine::Server::Notification::OutputChannelReplace::OutputChannelReplace(self, &Parameter).await;
			},

			"outputChannel.show" => {
				::Vine::Server::Notification::OutputChannelShow::OutputChannelShow(self, &Parameter).await;
			},

			"outputChannel.hide" => {
				::Vine::Server::Notification::OutputChannelHide::OutputChannelHide(self, &Parameter).await;
			},

			"outputChannel.dispose" => {
				::Vine::Server::Notification::OutputChannelDispose::OutputChannelDispose(self, &Parameter).await;
			},

			// Batch 13: webview reverse-channel (Mountain → renderer).
			"webview.postMessage" => {
				::Vine::Server::Notification::WebviewPostMessage::WebviewPostMessage(self, &Parameter).await;
			},

			"webview.dispose" => {
				::Vine::Server::Notification::WebviewDispose::WebviewDispose(self, &Parameter).await;
			},

			// Batch 14: grammar config, external-URI open, security alert.
			"set_language_configuration" => {
				::Vine::Server::Notification::SetLanguageConfiguration::SetLanguageConfiguration(self, &Parameter).await;
			},

			"openExternal" => {
				::Vine::Server::Notification::OpenExternal::OpenExternal(self, &Parameter).await;
			},

			"security.incident" => {
				::Vine::Server::Notification::SecurityIncident::SecurityIncident(self, &Parameter).await;
			},

			// Cocoon → Mountain: language-feature provider registration.
			// All 46+ `register_*` / `register_*_provider` variants delegate
			// to `Vine::Server::Notification::RegisterLanguageProvider` which
			// strips the prefix/suffix, logs, then calls
			// `VineHost::RegisterLanguageProvider` (Mountain's impl does the
			// type-name → ProviderType enum mapping and DTO construction).
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
				let _ = ::Vine::Server::Notification::RegisterLanguageProvider::RegisterLanguageProvider(
					self,
					&MethodName,
					&Parameter,
				)
				.await;
			},

			_ => {
				dev_log!("grpc", "[MountainVinegRPCService] Cocoon notification: {}", MethodName);

				// No typed match arm exists for this notification - it hits
				// the default path and becomes a `cocoon:<method>` Tauri
				// event that Wind may or may not listen for. The
				// `notif-drop` tag surfaces every fall-through so we can
				// tell at a glance which notifications Cocoon emits that
				// Mountain has no first-class handler for. The large OR
				// match above covers every `register_*` / `register_*_provider`
				// variant the Cocoon vscode-API shim is known to emit;
				// anything reaching here is either a new upstream addition or
				// an `unregister_*` / generic notification without a typed
				// handler. Payload preview included so diagnosis doesn't need
				// a second run.
				let PayloadPreview = if NotificationData.parameter.len() <= 160 {
					String::from_utf8_lossy(&NotificationData.parameter).into_owned()
				} else {
					let Slice = &NotificationData.parameter[..160];

					format!("{}…", String::from_utf8_lossy(Slice))
				};

				dev_log!(
					"notif-drop",
					"[NotifDrop] method={} payload_bytes={} preview={:?} (falls through to cocoon:{} event)",
					MethodName,
					NotificationData.parameter.len(),
					PayloadPreview,
					MethodName
				);

				// Forward all unknown notifications as Tauri events so Wind
				// can subscribe to any Cocoon-originated event.
				// Sanitize: Tauri only allows [a-zA-Z0-9\-/:_] in event names.
				// Dots → slashes (e.g. "webview.setOptions" → "webview/setOptions");
				// any other invalid char → "-".
				let SanitizedMethod:String = MethodName
					.chars()
					.map(|C| {
						match C {
							'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '/' | ':' | '_' => C,
							'.' => '/',
							_ => '-',
						}
					})
					.collect();
				let EventName = format!("cocoon:{}", SanitizedMethod);

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
