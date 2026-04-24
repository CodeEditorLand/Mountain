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
			// Batch 15: extension-host + progress + languages arms now live
			// as atoms under `Vine::Server::Notification::*`. Each match arm
			// is pure delegation - adding a new wire method is a one-line
			// change here plus one new atom file.
			"extensionHostMessage" => {
				super::Notification::ExtensionHostMessage::ExtensionHostMessage(self, &Parameter).await;
			},
			"ExtensionActivated" => {
				super::Notification::ExtensionActivated::ExtensionActivated(self, &Parameter).await;
			},
			"ExtensionDeactivated" => {
				super::Notification::ExtensionDeactivated::ExtensionDeactivated(self, &Parameter).await;
			},
			"WebviewReady" => {
				super::Notification::WebviewReady::WebviewReady(self, &Parameter).await;
			},
			"progress.start" => {
				super::Notification::ProgressStart::ProgressStart(self, &Parameter).await;
			},
			"progress.report" => {
				super::Notification::ProgressReport::ProgressReport(self, &Parameter).await;
			},
			"progress.end" => {
				super::Notification::ProgressEnd::ProgressEnd(self, &Parameter).await;
			},
			"languages.setDocumentLanguage" => {
				super::Notification::LanguagesSetDocumentLanguage::LanguagesSetDocumentLanguage(self, &Parameter).await;
			},
			"workspace.applyEdit" => {
				super::Notification::WorkspaceApplyEdit::WorkspaceApplyEdit(self, &Parameter).await;
			},
			"window.showTextDocument" => {
				super::Notification::WindowShowTextDocument::WindowShowTextDocument(self, &Parameter).await;
			},

			// Batch 16: the remaining Cocoon-notification arms, now pure
			// atom delegations. Each wire method lives in its own file
			// under `Vine::Server::Notification::*`. "Group atoms"
			// (TerminalLifecycle, DebugLifecycle, WebviewLifecycle, etc.)
			// handle 3-4 wire methods that share the same relay pattern.
			"webview.setTitle" | "webview.setIconPath" | "webview.setHtml" => {
				super::Notification::WebviewLifecycle::WebviewLifecycle(self, &MethodName, &Parameter).await;
			},
			"window.createTerminal" => {
				super::Notification::WindowCreateTerminal::WindowCreateTerminal(self, &Parameter).await;
			},
			"terminal.sendText" | "terminal.show" | "terminal.hide" | "terminal.dispose" => {
				super::Notification::TerminalLifecycle::TerminalLifecycle(self, &MethodName, &Parameter).await;
			},
			"window.createTextEditorDecorationType" | "window.disposeTextEditorDecorationType" => {
				super::Notification::DecorationTypeLifecycle::DecorationTypeLifecycle(self, &MethodName, &Parameter).await;
			},
			"debug.addBreakpoints" | "debug.removeBreakpoints" | "debug.consoleAppend" => {
				super::Notification::DebugLifecycle::DebugLifecycle(self, &MethodName, &Parameter).await;
			},
			"statusBar.update" | "statusBar.dispose" => {
				super::Notification::StatusBarLifecycle::StatusBarLifecycle(self, &MethodName, &Parameter).await;
			},
			"statusBar.message" => {
				super::Notification::StatusBarMessage::StatusBarMessage(self, &Parameter).await;
			},
			"window.showMessage" => {
				super::Notification::WindowShowMessage::WindowShowMessage(self, &Parameter).await;
			},
			"registerCommand" => {
				super::Notification::RegisterCommand::RegisterCommand(self, &Parameter).await;
			},
			"unregisterCommand" => {
				super::Notification::UnregisterCommand::UnregisterCommand(self, &Parameter).await;
			},

			// NOTE: `outputChannel.*` arms were previously here fanning to
			// the wrong `sky://output-channel/*` channel. Batch 9 atoms
			// below correctly route to `sky://output/*`; the legacy arm
			// was removed to stop it from shadowing the atoms.

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
