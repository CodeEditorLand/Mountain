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
//! - Safe error messages (no sensitive data)

use std::{collections::HashMap, sync::Arc};

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use ::Vine::Generated::{
	CancelOperationRequest,
	Empty,
	GenericNotification,
	GenericRequest,
	GenericResponse,
	RpcError as RPCError,
	mountain_service_server::MountainService,
};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track, dev_log};

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

	/// Registry of active operations with their cancellation tokens.
	/// Only populated for operations that explicitly opt into cancellation
	/// by calling RegisterOperation from within their handler body.
	/// Handlers MUST call UnregisterOperation on completion (success or error).
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

	/// Registers an operation for potential cancellation.
	/// Call this only from handlers that will also call UnregisterOperation
	/// on both the success and error completion paths.
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

	/// Unregisters an operation after completion.
	/// Must be called on both success and error paths for any operation
	/// that called RegisterOperation.
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
		if request.method.is_empty() {
			return Err(Status::invalid_argument("Method name cannot be empty"));
		}

		if request.method.len() > ServiceConfig::MAX_METHOD_NAME_LENGTH {
			return Err(Status::invalid_argument(format!(
				"Method name exceeds maximum length of {} characters",
				ServiceConfig::MAX_METHOD_NAME_LENGTH
			)));
		}

		if request.parameter.len() > 4 * 1024 * 1024 {
			return Err(Status::resource_exhausted("Request parameter size exceeds limit"));
		}

		Ok(())
	}

	/// Creates a JSON-RPC compliant error response.
	fn CreateErrorResponse(RequestIdentifier:u64, code:i32, message:String, data:Option<Vec<u8>>) -> GenericResponse {
		GenericResponse {
			request_identifier:RequestIdentifier,

			result:vec![],

			error:Some(RPCError { code, message, data:data.unwrap_or_default() }),
		}
	}

	/// Creates a successful JSON-RPC response.
	fn CreateSuccessResponse(RequestIdentifier:u64, result:&Value) -> GenericResponse {
		let result_bytes = match serde_json::to_vec(result) {
			Ok(bytes) => bytes,

			Err(e) => {
				dev_log!("grpc", "error: [MountainVinegRPCService] Failed to serialize result: {}", e);

				return Self::CreateErrorResponse(
					RequestIdentifier,
					-32603,
					"Failed to serialize response".to_string(),
					None,
				);
			},
		};

		GenericResponse { request_identifier:RequestIdentifier, result:result_bytes, error:None }
	}
}

/// Thin IPC provider that bridges Vine notifications to the gRPC client.
struct VineIPCProvider;

impl ::Vine::Host::IPCProvider for VineIPCProvider {
	fn SendRequest(
		&self,

		Channel:&str,

		Payload:serde_json::Value,
	) -> futures::future::BoxFuture<'_, ::Vine::Error::Result<serde_json::Value>> {
		let Channel = Channel.to_string();

		Box::pin(async move {
			crate::Vine::Client::SendRequest::Fn(&Channel, Channel.clone(), Payload, 10_000)
				.await
				.map_err(|E| ::Vine::Error::VineError::RPCError(format!("{:?}", E)))
		})
	}

	fn SendNotification(&self, Channel:&str, Method:&str, Payload:serde_json::Value) {
		let Channel = Channel.to_string();

		let Method = Method.to_string();

		tauri::async_runtime::spawn(async move {
			let _ = crate::Vine::Client::SendNotification::Fn(Channel, Method, Payload).await;
		});
	}
}

impl ::Vine::Host::ApplicationStateAccess for MountainVinegRPCService {
	fn EmbedderName(&self) -> &'static str { "Mountain" }
}

impl ::Vine::Host::VineHost for MountainVinegRPCService {
	fn ApplicationState(&self) -> &dyn ::Vine::Host::ApplicationStateAccess { self }

	fn EmitToRenderer(&self, Channel:&str, Payload:serde_json::Value) {
		let _ = self.ApplicationHandle.emit(Channel, Payload);
	}

	fn RendererEmitter(&self) -> Arc<dyn ::Vine::Host::RendererEmitter> {
		Arc::new(crate::Vine::Server::VineHostImpl::TauriRendererEmitter::New(
			self.ApplicationHandle.clone(),
		))
	}

	fn IPCProvider(&self) -> Arc<dyn ::Vine::Host::IPCProvider> { Arc::new(VineIPCProvider) }

	fn UnregisterProvider(&self, Handle:u32) {
		self.RunTime
			.Environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.LanguageProviders
			.lock()
			.remove(&Handle);
	}

	fn RegisterCommandInRegistry(&self, CommandId:&str, SideCarIdentifier:&str) {
		use tauri::Wry;

		use crate::Environment::CommandProvider::CommandHandler;

		self.RunTime.Environment.ApplicationState.Extension.Registry.RegisterCommand(
			CommandId.to_string(),
			CommandHandler::<Wry>::Proxied {
				SideCarIdentifier:SideCarIdentifier.to_string(),
				CommandIdentifier:CommandId.to_string(),
			},
		);
	}

	fn UnregisterCommandInRegistry(&self, CommandId:&str) {
		self.RunTime
			.Environment
			.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
			.remove(CommandId);
	}

	fn SpawnSendTextToTerminal(&self, TerminalId:u64, Text:String) {
		let RunTime = self.RunTime.clone();

		tauri::async_runtime::spawn(async move {
			let _ = crate::IPC::WindServiceHandlers::Terminal::TerminalSendText::Fn(
				RunTime,
				vec![serde_json::json!(TerminalId), serde_json::json!(Text)],
			)
			.await;
		});
	}

	fn SpawnDisposeTerminal(&self, TerminalId:u64) {
		let RunTime = self.RunTime.clone();

		tauri::async_runtime::spawn(async move {
			let _ = crate::IPC::WindServiceHandlers::Terminal::TerminalDispose::Fn(
				RunTime,
				vec![serde_json::json!(TerminalId)],
			)
			.await;
		});
	}

	fn CreateTerminal<'a>(
		&'a self,

		Options:&'a serde_json::Value,
	) -> futures::future::BoxFuture<'a, Option<serde_json::Value>> {
		use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

		let Options = Options.clone();

		let Env = self.RunTime.Environment.clone();

		Box::pin(async move {
			let Provider:Arc<dyn TerminalProvider> = Env.Require();

			Provider.CreateTerminal(Options).await.ok()
		})
	}

	fn RegisterScmInRegistry(&self, Handle:u32, ScmId:&str, Label:&str, _ExtId:&str) {
		use CommonLibrary::SourceControlManagement::DTO::SourceControlManagementProviderDTO::SourceControlManagementProviderDTO;

		let Dto = SourceControlManagementProviderDTO {
			Handle,

			Identifier:ScmId.to_string(),

			Label:Label.to_string(),

			RootURI:None,

			Count:None,

			CommitTemplate:None,

			AcceptInputCommand:None,

			InputBox:None,
		};

		self.RunTime
			.Environment
			.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementProviders
			.lock()
			.insert(Handle, Dto);
	}

	fn CreateSourceControl<'a>(&'a self, Payload:serde_json::Value) -> futures::future::BoxFuture<'a, ()> {
		use CommonLibrary::{
			Environment::Requires::Requires,
			SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
		};

		let Env = self.RunTime.Environment.clone();

		Box::pin(async move {
			let Provider:Arc<dyn SourceControlManagementProvider> = Env.Require();

			let _ = Provider.CreateSourceControl(Payload).await;
		})
	}

	fn UpdateSourceControlGroup<'a>(
		&'a self,

		ScmHandle:u32,

		Payload:serde_json::Value,
	) -> futures::future::BoxFuture<'a, ()> {
		use CommonLibrary::{
			Environment::Requires::Requires,
			SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
		};

		let Env = self.RunTime.Environment.clone();

		Box::pin(async move {
			let Provider:Arc<dyn SourceControlManagementProvider> = Env.Require();

			let _ = Provider.UpdateSourceControlGroup(ScmHandle, Payload).await;
		})
	}

	fn RegisterLanguageProvider(&self, Handle:u32, TypeName:&str, Payload:&serde_json::Value) -> bool {
		use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

		use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;

		let ProvType = match TypeName {
			"hover" => ProviderType::Hover,

			"completion" | "completion_item" => ProviderType::Completion,

			"signature_help" => ProviderType::SignatureHelp,

			"definition" => ProviderType::Definition,

			"reference" | "references" => ProviderType::References,

			"document_symbol" | "document_symbols" => ProviderType::DocumentSymbol,

			"workspace_symbol" | "workspace_symbols" => ProviderType::WorkspaceSymbol,

			"code_action" | "code_actions" => ProviderType::CodeAction,

			"code_lens" => ProviderType::CodeLens,

			"document_highlight" => ProviderType::DocumentHighlight,

			"document_formatting" => ProviderType::DocumentFormatting,

			"document_range_formatting" => ProviderType::DocumentRangeFormatting,

			"rename" => ProviderType::Rename,

			"folding_range" => ProviderType::FoldingRange,

			"selection_range" => ProviderType::SelectionRange,

			"semantic_tokens" => ProviderType::SemanticTokens,

			"inline_completion" | "inline_completions" => ProviderType::InlineCompletion,

			_ => return false,
		};

		let Selector = Payload
			.get("languageSelector")
			.or_else(|| Payload.get("language_selector"))
			.cloned()
			.unwrap_or(serde_json::json!([{"language":"*"}]));

		let ExtId = Payload
			.get("extensionId")
			.or_else(|| Payload.get("extension_id"))
			.cloned()
			.unwrap_or(serde_json::Value::Null);

		let Dto = ProviderRegistrationDTO {
			Handle,

			ProviderType:ProvType,

			Selector,

			SideCarIdentifier:"cocoon-main".to_string(),

			ExtensionIdentifier:ExtId,

			Options:None,
		};

		self.RunTime
			.Environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.RegisterProvider(Handle, Dto);

		true
	}

	fn UpdateScmGroupMarkers(&self, ScmHandle:u32, GroupId:&str, ResourceStates:&serde_json::Value) {
		crate::Vine::Server::VineHostImpl::UpdateScmGroupMarkers(&self.RunTime, ScmHandle, GroupId, ResourceStates);
	}
}

#[tonic::async_trait]
impl MountainService for MountainVinegRPCService {
	type OpenChannelFromCocoonStream = std::pin::Pin<
		Box<
			dyn tonic::codegen::tokio_stream::Stream<Item = Result<::Vine::Generated::Envelope, tonic::Status>>
				+ Send
				+ 'static,
		>,
	>;

	async fn open_channel_from_cocoon(
		&self,

		_request:tonic::Request<tonic::Streaming<::Vine::Generated::Envelope>>,
	) -> Result<tonic::Response<Self::OpenChannelFromCocoonStream>, tonic::Status> {
		Err(tonic::Status::unimplemented(
			"OpenChannelFromCocoon: streaming multiplexer not yet wired (Patch 14); use unary endpoints",
		))
	}

	/// Handles generic request-response RPCs from Cocoon.
	///
	/// Operations that require cancellation support must call RegisterOperation
	/// from within their handler and UnregisterOperation on completion.
	/// process_cocoon_request itself does NOT register operations
	/// unconditionally to avoid an unbounded ActiveOperations map (one leaked
	/// entry per RPC).
	async fn process_cocoon_request(
		&self,

		request:Request<GenericRequest>,
	) -> Result<Response<GenericResponse>, Status> {
		let RequestData = request.into_inner();

		let MethodName = RequestData.method.clone();

		let RequestIdentifier = RequestData.request_identifier;

		let ReceiveInstant = std::time::Instant::now();

		dev_log!(
			"grpc-verbose",
			"[MountainVinegRPCService] recv id={} method={} size={}B",
			RequestIdentifier,
			MethodName,
			RequestData.parameter.len()
		);

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
				"rpc-latency",
				"[LandFix:RPC] grpc-recv method={} id={} size={} t_ns={}",
				MethodName,
				RequestIdentifier,
				RequestData.parameter.len(),
				InstrumentRecvNs
			);
		}

		if let Err(status) = self.ValidateRequest(&RequestData) {
			dev_log!("grpc", "warn: [MountainVinegRPCService] Request validation failed: {}", status);

			return Ok(Response::new(Self::CreateErrorResponse(
				RequestIdentifier,
				-32602,
				status.message().to_string(),
				None,
			)));
		}

		let ParametersValue:Value = match serde_json::from_slice(&RequestData.parameter) {
			Ok(v) => v,

			Err(e) => {
				let msg = format!("Failed to deserialize parameters for method '{}': {}", MethodName, e);

				dev_log!("grpc", "error: {}", msg);

				return Ok(Response::new(Self::CreateErrorResponse(RequestIdentifier, -32700, msg, None)));
			},
		};

		let DispatchResult = Track::SideCarRequest::DispatchSideCarRequest::DispatchSideCarRequest(
			self.ApplicationHandle.clone(),
			self.RunTime.clone(),
			"cocoon-main",
			MethodName.clone(),
			ParametersValue,
		)
		.await;

		match DispatchResult {
			Ok(SuccessfulResult) => {
				if IsHotRpc {
					dev_log!(
						"rpc-latency",
						"[LandFix:RPC] dispatched method={} id={} elapsed={}ms",
						MethodName,
						RequestIdentifier,
						ReceiveInstant.elapsed().as_millis()
					);
				}

				dev_log!(
					"grpc-verbose",
					"[MountainVinegRPCService] Request [ID: {}] completed successfully",
					RequestIdentifier
				);

				Ok(Response::new(Self::CreateSuccessResponse(RequestIdentifier, &SuccessfulResult)))
			},

			Err(ErrorString) => {
				let LowerError = ErrorString.to_lowercase();

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

	async fn send_cocoon_notification(&self, request:Request<GenericNotification>) -> Result<Response<Empty>, Status> {
		let NotificationData = request.into_inner();

		let MethodName = NotificationData.method;

		dev_log!(
			"grpc-verbose",
			"[MountainVinegRPCService] Received gRPC Notification: Method='{}'",
			MethodName
		);

		if MethodName.is_empty() {
			dev_log!(
				"grpc",
				"warn: [MountainVinegRPCService] Received notification with empty method name"
			);

			return Err(Status::invalid_argument("Method name cannot be empty"));
		}

		let Parameter:Value = if NotificationData.parameter.is_empty() {
			Value::Null
		} else {
			serde_json::from_slice(&NotificationData.parameter).unwrap_or(Value::Null)
		};

		match MethodName.as_str() {
			"extensionHostMessage" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"cocoon:extensionHostReply",
					&Parameter,
					"",
					"",
				);
			},

			"ExtensionActivated" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"cocoon:extensionActivated",
					&Parameter,
					"",
					"",
				);
			},

			"ExtensionDeactivated" => {
				dev_log!(
					"grpc",
					"[Extension] deactivated id={}",
					Parameter.get("extensionId").and_then(Value::as_str).unwrap_or("?")
				);
			},

			"WebviewReady" => {
				dev_log!(
					"grpc",
					"[Webview] ready handle={}",
					Parameter.get("handle").and_then(Value::as_str).unwrap_or("?")
				);
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
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://languages/setDocumentLanguage",
					&Parameter,
					"grpc",
					"",
				);
			},

			"workspace.applyEdit" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://workspace/applyEdit",
					&Parameter,
					"",
					"",
				);
			},

			"window.showTextDocument" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://window/showTextDocument",
					&Parameter,
					"",
					"",
				);
			},

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

			"tree.refresh" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://tree-view/refresh",
					&Parameter,
					"grpc",
					"[Tree] refresh",
				);
			},

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
				::Vine::Server::Notification::DecorationTypeLifecycle::DecorationTypeLifecycle(
					self,
					&MethodName,
					&Parameter,
				)
				.await;
			},

			"window.setTextEditorDecorations" => {
				::Vine::Server::Notification::SetTextEditorDecorations::SetTextEditorDecorations(self, &Parameter)
					.await;
			},

			"window.applyTextEdits" => {
				::Vine::Server::Notification::ApplyTextEdits::ApplyTextEdits(self, &Parameter).await;
			},

			"debug.addBreakpoints" | "debug.removeBreakpoints" | "debug.consoleAppend" => {
				::Vine::Server::Notification::DebugLifecycle::DebugLifecycle(self, &MethodName, &Parameter).await;
			},

			"statusBar.update" | "statusBar.dispose" => {
				::Vine::Server::Notification::StatusBarLifecycle::StatusBarLifecycle(self, &MethodName, &Parameter)
					.await;
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

			"unregister_file_system_provider" => {
				dev_log!(
					"provider-register",
					"[ProviderUnregister] file_system scheme={}",
					Parameter.get("scheme").and_then(Value::as_str).unwrap_or("")
				);

				::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(
					self,
					&Parameter,
					"file_system",
				);
			},

			"unregister_scm_provider" => {
				::Vine::Server::Notification::UnregisterScmProvider::UnregisterScmProvider(self, &Parameter).await;
			},

			"unregister_uri_handler" => {
				dev_log!(
					"provider-register",
					"[ProviderUnregister] uri_handler scheme={}",
					Parameter.get("scheme").and_then(Value::as_str).unwrap_or("")
				);

				::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(
					self,
					&Parameter,
					"uri_handler",
				);
			},

			"update_scm_group" => {
				::Vine::Server::Notification::UpdateScmGroup::UpdateScmGroup(self, &Parameter).await;
			},

			"register_scm_provider" => {
				::Vine::Server::Notification::RegisterScmProvider::RegisterScmProvider(self, &Parameter).await;
			},

			"register_scm_resource_group" => {
				::Vine::Server::Notification::RegisterScmResourceGroup::RegisterScmResourceGroup(self, &Parameter)
					.await;
			},

			"progress.update" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://notification/progress-update",
					&Parameter,
					"grpc",
					"[Progress] update",
				);
			},

			"progress.complete" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://progress/complete",
					&Parameter,
					"grpc",
					"[Progress] complete",
				);
			},

			"setStatusBarText" => {
				::Vine::Server::Notification::SetStatusBarText::SetStatusBarText(self, &Parameter).await;
			},

			"disposeStatusBarItem" => {
				::Vine::Server::Notification::DisposeStatusBarItem::DisposeStatusBarItem(self, &Parameter).await;
			},

			"output.create" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/create",
					&Parameter,
					"grpc",
					"[Output] create",
				);
			},

			"output.append" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/append",
					&Parameter,
					"grpc",
					"[Output] append",
				);
			},

			"output.appendLine" => {
				::Vine::Server::Notification::OutputAppendLine::OutputAppendLine(self, &Parameter).await;
			},

			"output.clear" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/clear",
					&Parameter,
					"grpc",
					"[Output] clear",
				);
			},

			"output.show" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/show",
					&Parameter,
					"grpc",
					"[Output] show",
				);
			},

			"output.dispose" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/dispose",
					&Parameter,
					"grpc",
					"[Output] dispose",
				);
			},

			"output.replace" => {
				::Vine::Server::Notification::OutputReplace::OutputReplace(self, &Parameter).await;
			},

			"outputChannel.create" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/create",
					&Parameter,
					"output-verbose",
					"[OutputChannel] create",
				);
			},

			"outputChannel.append" => {
				::Vine::Server::Notification::OutputChannelAppend::OutputChannelAppend(self, &Parameter).await;
			},

			"outputChannel.clear" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/clear",
					&Parameter,
					"grpc",
					"[OutputChannel] clear",
				);
			},

			"outputChannel.replace" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/replace",
					&Parameter,
					"grpc",
					"[OutputChannel] replace",
				);
			},

			"outputChannel.show" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/show",
					&Parameter,
					"grpc",
					"[OutputChannel] show",
				);
			},

			"outputChannel.hide" => {
				::Vine::Server::Notification::OutputChannelHide::OutputChannelHide(self, &Parameter).await;
			},

			"outputChannel.dispose" => {
				::Vine::Server::Notification::Support::RelayToSky::Fn(
					self,
					"sky://output/dispose",
					&Parameter,
					"grpc",
					"[OutputChannel] dispose",
				);
			},

			"webview.postMessage" => {
				::Vine::Server::Notification::WebviewPostMessage::WebviewPostMessage(self, &Parameter).await;
			},

			"webview.dispose" => {
				::Vine::Server::Notification::WebviewDispose::WebviewDispose(self, &Parameter).await;
			},

			"set_language_configuration" => {
				::Vine::Server::Notification::SetLanguageConfiguration::SetLanguageConfiguration(self, &Parameter)
					.await;
			},

			"openExternal" => {
				::Vine::Server::Notification::OpenExternal::OpenExternal(self, &Parameter).await;
			},

			"security.incident" => {
				::Vine::Server::Notification::SecurityIncident::SecurityIncident(self, &Parameter).await;
			},

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

				let PayloadPreview = if NotificationData.parameter.len() <= 160 {
					String::from_utf8_lossy(&NotificationData.parameter).into_owned()
				} else {
					let Slice = &NotificationData.parameter[..160];

					format!("{}...", String::from_utf8_lossy(Slice))
				};

				dev_log!(
					"notif-drop",
					"[NotifDrop] method={} payload_bytes={} preview={:?} (falls through to cocoon:{} event)",
					MethodName,
					NotificationData.parameter.len(),
					PayloadPreview,
					MethodName
				);

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

	async fn cancel_operation(&self, request:Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		let cancel_request = request.into_inner();

		let RequestIdentifierToCancel = cancel_request.request_identifier_to_cancel;

		dev_log!(
			"grpc",
			"[MountainVinegRPCService] Received CancelOperation request for RequestID: {}",
			RequestIdentifierToCancel
		);

		let cancel_token = {
			let operations = self.ActiveOperations.read().await;

			operations.get(&RequestIdentifierToCancel).cloned()
		};

		match cancel_token {
			Some(token) => {
				token.cancel();

				self.ActiveOperations.write().await.remove(&RequestIdentifierToCancel);

				dev_log!(
					"grpc",
					"[MountainVinegRPCService] Successfully initiated cancellation for operation {}",
					RequestIdentifierToCancel
				);

				Ok(Response::new(Empty {}))
			},

			None => {
				dev_log!(
					"grpc",
					"warn: [MountainVinegRPCService] Cannot cancel operation {}: operation not found (may have \
					 already completed)",
					RequestIdentifierToCancel
				);

				Ok(Response::new(Empty {}))
			},
		}
	}
}
