// # CocoonServiceImpl - thin-wrapper dispatcher
//
// Domain files hold all typed RPC implementations. This module keeps:
// - CocoonServiceImpl struct + helper methods
// - process_mountain_request (legacy generic router, ~600 lines)
// - send_mountain_notification (push dispatcher, ~400 lines)
// - One-line delegates for all 78 typed RPCs

pub mod Auth;

pub mod GenericNotification;

pub mod GenericRequest;

pub mod Command;

pub mod Debug;

pub mod Extension;

pub mod FileSystem;

pub mod Initialization;

pub mod Output;

pub mod Provider;

pub mod Save;

pub mod SCM;

pub mod Secret;

pub mod Task;

pub mod Terminal;

pub mod TreeView;

pub mod Window;

pub mod Workspace;

use std::{
	collections::HashMap,
	sync::{
		Arc,
		atomic::{AtomicU64, Ordering},
	},
	time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	LanguageFeature::{
		DTO::{PositionDTO::PositionDTO, ProviderType::ProviderType},
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
	Secret::SecretProvider::SecretProvider,
	Terminal::TerminalProvider::TerminalProvider,
	UserInterface::{
		DTO::{
			InputBoxOptionsDTO::InputBoxOptionsDTO,
			QuickPickItemDTO::QuickPickItemDTO,
			QuickPickOptionsDTO::QuickPickOptionsDTO,
		},
		UserInterfaceProvider::UserInterfaceProvider,
	},
};
use serde_json::json;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use url::Url;
use ::Vine::Generated::cocoon_service_server::CocoonService;
use ::Vine::Generated::{
	// Service trait
	// Extended Language + Window + FS + Output + Task + Auth + Debug + Extension types
	AppendOutputRequest,
	ApplyEditRequest,
	ApplyEditResponse,
	Argument,
	CancelOperationRequest,

	ClearOutputRequest,
	CloseTerminalRequest,
	CodeAction,

	CompletionItem,
	CopyFileRequest,
	CreateDirectoryRequest,
	CreateOutputChannelRequest,
	CreateOutputChannelResponse,
	CreateStatusBarItemRequest,
	CreateStatusBarItemResponse,
	CreateWebviewPanelRequest,
	CreateWebviewPanelResponse,
	DebugConfiguration,
	DeleteFileRequest,
	DeleteSecretRequest,
	DisposeOutputRequest,
	DisposeWebviewPanelRequest,
	// Common types
	Empty,
	ExecuteCommandRequest,
	ExecuteCommandResponse,
	ExecuteTaskRequest,
	ExecuteTaskResponse,
	ExtensionInfo,
	// Workspace Operations
	FindFilesRequest,
	FindFilesResponse,
	FindTextInFilesRequest,
	FindTextInFilesResponse,
	GenericNotification as GenericNotificationMsg,
	// Common generic types
	GenericRequest as GenericRequestMsg,
	GenericResponse,
	GetAllExtensionsResponse,
	GetAuthenticationSessionRequest,
	GetAuthenticationSessionResponse,
	GetConfigurationRequest,
	GetConfigurationResponse,
	GetExtensionRequest,
	GetExtensionResponse,
	// Secret Storage
	GetSecretRequest,
	GetSecretResponse,
	GetTreeChildrenRequest,
	GetTreeChildrenResponse,
	GitExecRequest,
	GitExecResponse,

	// Initialization
	InitExtensionHostRequest,

	Location,
	OnDidReceiveMessageRequest,

	OpenDocumentRequest,
	OpenDocumentResponse,
	OpenExternalRequest,
	// Terminal
	OpenTerminalRequest,
	// Save Participants
	ParticipateInSaveRequest,
	ParticipateInSaveResponse,
	Position,
	PostWebviewMessageRequest,
	ProvideCallHierarchyRequest,
	ProvideCallHierarchyResponse,
	ProvideCodeActionsRequest,
	ProvideCodeActionsResponse,
	ProvideCodeLensesRequest,
	ProvideCodeLensesResponse,
	ProvideCompletionItemsRequest,
	ProvideCompletionItemsResponse,
	ProvideDefinitionRequest,
	ProvideDefinitionResponse,
	ProvideDocumentFormattingRequest,
	ProvideDocumentFormattingResponse,
	ProvideDocumentHighlightsRequest,
	ProvideDocumentHighlightsResponse,
	ProvideDocumentRangeFormattingRequest,
	ProvideDocumentRangeFormattingResponse,
	ProvideDocumentSymbolsRequest,
	ProvideDocumentSymbolsResponse,
	ProvideFoldingRangesRequest,
	ProvideFoldingRangesResponse,
	ProvideHoverRequest,
	ProvideHoverResponse,
	ProvideInlayHintsRequest,
	ProvideInlayHintsResponse,
	ProvideInlineCompletionRequest,
	ProvideInlineCompletionResponse,
	ProvideLinkedEditingRangesRequest,
	ProvideLinkedEditingRangesResponse,
	ProvideOnTypeFormattingRequest,
	ProvideOnTypeFormattingResponse,
	ProvideReferencesRequest,
	ProvideReferencesResponse,
	ProvideRenameEditsRequest,
	ProvideRenameEditsResponse,
	ProvideSelectionRangesRequest,
	ProvideSelectionRangesResponse,
	ProvideSemanticTokensRequest,
	ProvideSemanticTokensResponse,
	ProvideSignatureHelpRequest,
	ProvideSignatureHelpResponse,
	ProvideTypeHierarchyRequest,
	ProvideTypeHierarchyResponse,
	ProvideWorkspaceSymbolsRequest,
	ProvideWorkspaceSymbolsResponse,
	Range,
	// File System
	ReadFileRequest,
	ReadFileResponse,
	ReaddirRequest,
	ReaddirResponse,
	RegisterAuthenticationProviderRequest,
	// Commands
	RegisterCommandRequest,
	// Debug
	RegisterDebugAdapterRequest,
	RegisterOnTypeFormattingProviderRequest,
	// Language Features
	RegisterProviderRequest,
	// SCM
	RegisterScmProviderRequest,
	RegisterSemanticTokensProviderRequest,
	RegisterSignatureHelpProviderRequest,
	RegisterTaskProviderRequest,
	// Tree View
	RegisterTreeViewProviderRequest,
	RenameFileRequest,
	ReportProgressRequest,
	ResizeTerminalRequest,
	RpcError,
	SaveAllRequest,
	SaveAllResponse,
	SetStatusBarTextRequest,
	SetWebviewHtmlRequest,
	ShowInputBoxRequest,
	ShowInputBoxResponse,
	ShowMessageRequest,
	ShowMessageResponse,
	ShowOutputRequest,
	ShowProgressRequest,
	ShowProgressResponse,
	ShowQuickPickRequest,
	ShowQuickPickResponse,
	// Window Operations
	ShowTextDocumentRequest,
	ShowTextDocumentResponse,
	SourceControlResourceState,
	StartDebuggingRequest,
	StartDebuggingResponse,

	StatRequest,
	StatResponse,
	StopDebuggingRequest,
	StoreSecretRequest,
	TerminalClosedNotification,
	TerminalDataNotification,

	TerminalInputRequest,
	TerminalOpenedNotification,
	TerminalProcessIdNotification,
	TerminateTaskRequest,
	TextDocumentSaveReason,
	TextEdit,
	TextEditForSave,

	TextMatch,
	TreeItem,

	UnregisterCommandRequest,

	UpdateConfigurationRequest,
	UpdateScmGroupRequest,
	UpdateWorkspaceFoldersRequest,

	Uri,
	ViewColumn,

	WatchFileRequest,

	WorkspaceFolder,
	WriteFileRequest,

	on_did_receive_message_request,
	post_webview_message_request,
};

use dashmap::DashMap;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
	ApplicationState::DTO::{
		ProviderRegistrationDTO::ProviderRegistrationDTO,
		WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	},
	Environment::MountainEnvironment::MountainEnvironment,
};
// Import generated protobuf types
use crate::dev_log;

/// Monotonic counter for outbound channel slots in `CHANNEL_REGISTRY`.
static NEXT_CHANNEL_ID:AtomicU64 = AtomicU64::new(1);

lazy_static! {
	/// Process-wide map from channel_id → outbound `Envelope` sender.
	/// Callers that hold a channel_id can push frames back to the Cocoon
	/// peer over the bidirectional stream without touching the original
	/// `open_channel_from_mountain` future.
	static ref CHANNEL_REGISTRY:DashMap<u64, mpsc::Sender<::Vine::Generated::Envelope>> =
		DashMap::new();
}

/// Implementation of the CocoonService gRPC server
///
/// This struct handles all incoming requests from the Cocoon extension host
/// sidecar and dispatches them to the appropriate Mountain services.
#[derive(Clone)]
pub struct CocoonServiceImpl {
	/// Mountain environment providing access to all services
	environment:Arc<MountainEnvironment>,

	/// Registry of active operations with their cancellation tokens
	/// Maps request ID to cancellation token for operation cancellation
	ActiveOperations:Arc<RwLock<HashMap<u64, tokio_util::sync::CancellationToken>>>,
}

impl CocoonServiceImpl {
	/// Creates a new instance of the CocoonService server
	///
	/// # Parameters
	/// - `environment`: Mountain environment with access to all services
	///
	/// # Returns
	/// A new CocoonService instance
	pub fn new(environment:Arc<MountainEnvironment>) -> Self {
		dev_log!("cocoon", "[CocoonService] New instance created");

		Self { environment, ActiveOperations:Arc::new(RwLock::new(HashMap::new())) }
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

		dev_log!("cocoon", "[CocoonService] Registered operation {} for cancellation", request_id);

		token
	}

	/// Unregisters an operation after completion
	///
	/// # Parameters
	/// - `request_id`: The request identifier to unregister
	pub async fn UnregisterOperation(&self, request_id:u64) {
		self.ActiveOperations.write().await.remove(&request_id);

		dev_log!("cocoon", "[CocoonService] Unregistered operation {}", request_id);
	}

	/// Registers a language feature provider in ApplicationState.
	///
	/// Converts the gRPC request fields into a `ProviderRegistrationDTO` and
	/// stores it in `ApplicationState.Extension.ProviderRegistration`.
	///
	/// # Parameters
	/// - `handle`: Unique provider handle
	/// - `provider_type`: The type of language feature
	/// - `language_selector`: Language scope (e.g. "typescript")
	/// - `extension_id`: Extension that registered this provider
	fn RegisterProvider(&self, handle:u32, provider_type:ProviderType, language_selector:&str, extension_id:&str) {
		// SideCarIdentifier = "cocoon-main" so FeatureMethods::invoke_provider can
		// route back via Vine::Client::SendRequestToSideCar("cocoon-main", ...).
		// Selector stored as array so ProviderLookup::get_matching_provider's
		// `.as_array()` call finds the language entry: [{ "language": "typescript" }].
		let dto = ProviderRegistrationDTO {
			Handle:handle,

			ProviderType:provider_type,

			Selector:json!([{ "language": language_selector }]),

			SideCarIdentifier:"cocoon-main".to_string(),

			ExtensionIdentifier:json!(extension_id),

			Options:None,
		};

		self.environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.RegisterProvider(handle, dto);

		dev_log!(
			"cocoon",
			"[CocoonService] Provider {:?} registered: handle={}, language={}",
			provider_type,
			handle,
			language_selector
		);
	}

	/// Extracts a filesystem path from a URI proto message.
	///
	/// Handles both `file://` URIs and bare paths. Returns `None` if the URI
	/// is absent or the path cannot be extracted.
	fn UriToPath(uri_opt:Option<&Uri>) -> Option<std::path::PathBuf> {
		let value = uri_opt?.value.as_str();

		if value.is_empty() {
			return None;
		}

		// Strip file:// prefix if present
		let path_str = if let Some(Stripped) = value.strip_prefix("file://") {
			Stripped
		} else if value.starts_with('/') || (value.len() > 1 && value.as_bytes()[1] == b':') {
			// Bare absolute path (Unix or Windows)
			value
		} else {
			// Unknown scheme - return as-is
			value
		};

		Some(std::path::PathBuf::from(path_str))
	}
}

#[async_trait]

impl CocoonService for CocoonServiceImpl {
	// Bidirectional streaming channel. Cocoon opens this stream and sends
	// Envelope frames; Mountain reads each frame and routes:
	//   Notification  → GenericNotification::Dispatcher (same path as the unary endpoint)
	//   Request       → GenericRequest::Dispatcher; response sent back via out_tx
	//   Response      → log and ignore (not expected on this direction)
	//   Cancel        → no-op
	// Outbound frames are delivered via the mpsc channel whose sender is
	// stored in CHANNEL_REGISTRY keyed by a per-call channel_id.
	type OpenChannelFromMountainStream = std::pin::Pin<
		Box<
			dyn tonic::codegen::tokio_stream::Stream<Item = Result<::Vine::Generated::Envelope, tonic::Status>>
				+ Send
				+ 'static,
		>,
	>;

	async fn open_channel_from_mountain(
		&self,

		request:tonic::Request<tonic::Streaming<::Vine::Generated::Envelope>>,
	) -> Result<tonic::Response<Self::OpenChannelFromMountainStream>, tonic::Status> {
		use futures_util::StreamExt;
		use ::Vine::Generated::envelope::Payload;
		use ::Vine::Generated::{Envelope, GenericResponse, RpcError};

		let ChannelId = NEXT_CHANNEL_ID.fetch_add(1, Ordering::Relaxed);

		let (OutTx, OutRx) = mpsc::channel::<Envelope>(1024);

		// Register the outbound sender so other subsystems can push frames
		// to this stream using the channel_id.
		CHANNEL_REGISTRY.insert(ChannelId, OutTx.clone());

		dev_log!("cocoon", "[CocoonService] open_channel_from_mountain channel_id={}", ChannelId);

		let mut Inbound = request.into_inner();

		let ServiceClone = self.clone();

		// Spawn the read pump as a detached task so the method returns the
		// outbound stream immediately and Cocoon can begin receiving frames.
		tauri::async_runtime::spawn(async move {
			while let Some(FrameResult) = Inbound.next().await {
				let Frame = match FrameResult {
					Ok(F) => F,

					Err(Status) => {
						dev_log!(
							"cocoon",
							"[CocoonService] channel_id={} inbound error: {}",
							ChannelId,
							Status
						);

						break;
					},
				};

				let Payload = match Frame.payload {
					Some(P) => P,

					None => continue,
				};

				match Payload {
					Payload::Notification(N) => {
						// Reuse the unary notification dispatcher verbatim.
						let _ = GenericNotification::Dispatcher::Fn(
							&ServiceClone,
							tonic::Request::new(N),
						)
						.await;
					},

					Payload::Request(R) => {
						let RequestId = R.request_identifier;

						// Reuse the unary request dispatcher; wrap the result
						// into a Response envelope and push it back.
						let Wrapped = tonic::Request::new(R);

						let Response = match GenericRequest::Dispatcher::Fn(&ServiceClone, Wrapped).await {
							Ok(GrpcResponse) => {
								let Inner = GrpcResponse.into_inner();

								Envelope {
									payload:Some(Payload::Response(Inner)),
								}
							},

							Err(Status) => Envelope {
								payload:Some(Payload::Response(GenericResponse {
									request_identifier:RequestId,
									result:Vec::new(),
									error:Some(RpcError {
										code:Status.code() as i32,
										message:Status.message().to_string(),
										data:Vec::new(),
									}),
								})),
							},
						};

						if OutTx.send(Response).await.is_err() {
							// Receiver closed - peer disconnected.
							break;
						}
					},

					Payload::Response(_) => {
						// Responses on the Mountain-inbound direction are
						// unexpected; drop silently.
						dev_log!(
							"cocoon",
							"[CocoonService] channel_id={} unexpected Response frame; ignored",
							ChannelId
						);
					},

					Payload::Cancel(_) => {
						// Best-effort cancel; the unary path has no cancel
						// support so this is a deliberate no-op.
					},
				}
			}

			// Pump exited - remove the registry entry so stale senders are
			// not retained.
			CHANNEL_REGISTRY.remove(&ChannelId);

			dev_log!("cocoon", "[CocoonService] open_channel_from_mountain channel_id={} closed", ChannelId);
		});

		let OutboundStream = ReceiverStream::new(OutRx).map(|E| Ok(E));

		Ok(tonic::Response::new(Box::pin(OutboundStream)))
	}

	/// Process Mountain requests from Cocoon (generic request-response).
	/// Implementation in `GenericRequest::Dispatcher`.
	async fn process_mountain_request(
		&self,

		request:Request<GenericRequestMsg>,
	) -> Result<Response<GenericResponse>, Status> {
		return GenericRequest::Dispatcher::Fn(self, request).await;
	}

	/// Send Mountain notifications to Cocoon (generic fire-and-forget).
	/// Implementation in `GenericNotification::Dispatcher`.
	async fn send_mountain_notification(
		&self,

		request:Request<GenericNotificationMsg>,
	) -> Result<Response<Empty>, Status> {
		return GenericNotification::Dispatcher::Fn(self, request).await;
	}

	async fn cancel_operation(&self, request:Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		Initialization::CancelOperation::Fn(self, request.into_inner()).await
	}

	async fn initial_handshake(&self, request:Request<Empty>) -> Result<Response<Empty>, Status> {
		Initialization::InitialHandshake::Fn(self, request.into_inner()).await
	}

	async fn init_extension_host(&self, request:Request<InitExtensionHostRequest>) -> Result<Response<Empty>, Status> {
		Initialization::InitExtensionHost::Fn(self, request.into_inner()).await
	}

	async fn register_command(&self, request:Request<RegisterCommandRequest>) -> Result<Response<Empty>, Status> {
		Command::RegisterCommand::Fn(self, request.into_inner()).await
	}

	async fn execute_contributed_command(
		&self,

		request:Request<ExecuteCommandRequest>,
	) -> Result<Response<ExecuteCommandResponse>, Status> {
		Command::ExecuteContributedCommand::Fn(self, request.into_inner()).await
	}

	async fn unregister_command(&self, request:Request<UnregisterCommandRequest>) -> Result<Response<Empty>, Status> {
		Command::UnregisterCommand::Fn(self, request.into_inner()).await
	}

	async fn register_hover_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterHoverProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_hover(
		&self,

		request:Request<ProvideHoverRequest>,
	) -> Result<Response<ProvideHoverResponse>, Status> {
		Provider::ProvideHover::Fn(self, request.into_inner()).await
	}

	async fn register_completion_item_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterCompletionItemProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_completion_items(
		&self,

		request:Request<ProvideCompletionItemsRequest>,
	) -> Result<Response<ProvideCompletionItemsResponse>, Status> {
		Provider::ProvideCompletionItems::Fn(self, request.into_inner()).await
	}

	async fn register_definition_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterDefinitionProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_definition(
		&self,

		request:Request<ProvideDefinitionRequest>,
	) -> Result<Response<ProvideDefinitionResponse>, Status> {
		Provider::ProvideDefinition::Fn(self, request.into_inner()).await
	}

	async fn register_reference_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterReferenceProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_references(
		&self,

		request:Request<ProvideReferencesRequest>,
	) -> Result<Response<ProvideReferencesResponse>, Status> {
		Provider::ProvideReferences::Fn(self, request.into_inner()).await
	}

	async fn register_code_actions_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterCodeActionsProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_code_actions(
		&self,

		request:Request<ProvideCodeActionsRequest>,
	) -> Result<Response<ProvideCodeActionsResponse>, Status> {
		Provider::ProvideCodeActions::Fn(self, request.into_inner()).await
	}

	async fn show_text_document(
		&self,

		request:Request<ShowTextDocumentRequest>,
	) -> Result<Response<ShowTextDocumentResponse>, Status> {
		Window::ShowTextDocument::Fn(self, request.into_inner()).await
	}

	async fn show_information_message(
		&self,

		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		Window::ShowInformationMessage::Fn(self, request.into_inner()).await
	}

	async fn show_warning_message(
		&self,

		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		Window::ShowWarningMessage::Fn(self, request.into_inner()).await
	}

	async fn show_error_message(
		&self,

		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		Window::ShowErrorMessage::Fn(self, request.into_inner()).await
	}

	async fn create_status_bar_item(
		&self,

		request:Request<CreateStatusBarItemRequest>,
	) -> Result<Response<CreateStatusBarItemResponse>, Status> {
		Window::CreateStatusBarItem::Fn(self, request.into_inner()).await
	}

	async fn set_status_bar_text(&self, request:Request<SetStatusBarTextRequest>) -> Result<Response<Empty>, Status> {
		Window::SetStatusBarText::Fn(self, request.into_inner()).await
	}

	async fn create_webview_panel(
		&self,

		request:Request<CreateWebviewPanelRequest>,
	) -> Result<Response<CreateWebviewPanelResponse>, Status> {
		Window::CreateWebviewPanel::Fn(self, request.into_inner()).await
	}

	async fn set_webview_html(&self, request:Request<SetWebviewHtmlRequest>) -> Result<Response<Empty>, Status> {
		Window::SetWebviewHtml::Fn(self, request.into_inner()).await
	}

	async fn on_did_receive_message(
		&self,

		request:Request<OnDidReceiveMessageRequest>,
	) -> Result<Response<Empty>, Status> {
		Window::OnDidReceiveMessage::Fn(self, request.into_inner()).await
	}

	async fn post_webview_message(
		&self,

		request:Request<PostWebviewMessageRequest>,
	) -> Result<Response<Empty>, Status> {
		Window::PostWebviewMessage::Fn(self, request.into_inner()).await
	}

	async fn dispose_webview_panel(
		&self,

		request:Request<DisposeWebviewPanelRequest>,
	) -> Result<Response<Empty>, Status> {
		Window::DisposeWebviewPanel::Fn(self, request.into_inner()).await
	}

	async fn read_file(&self, request:Request<ReadFileRequest>) -> Result<Response<ReadFileResponse>, Status> {
		FileSystem::ReadFile::Fn(self, request.into_inner()).await
	}

	async fn write_file(&self, request:Request<WriteFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::WriteFile::Fn(self, request.into_inner()).await
	}

	async fn stat(&self, request:Request<StatRequest>) -> Result<Response<StatResponse>, Status> {
		FileSystem::Stat::Fn(self, request.into_inner()).await
	}

	async fn readdir(&self, request:Request<ReaddirRequest>) -> Result<Response<ReaddirResponse>, Status> {
		FileSystem::Readdir::Fn(self, request.into_inner()).await
	}

	async fn watch_file(&self, request:Request<WatchFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::WatchFile::Fn(self, request.into_inner()).await
	}

	async fn find_files(&self, request:Request<FindFilesRequest>) -> Result<Response<FindFilesResponse>, Status> {
		FileSystem::FindFiles::Fn(self, request.into_inner()).await
	}

	async fn find_text_in_files(
		&self,

		request:Request<FindTextInFilesRequest>,
	) -> Result<Response<FindTextInFilesResponse>, Status> {
		FileSystem::FindTextInFiles::Fn(self, request.into_inner()).await
	}

	async fn delete_file(&self, request:Request<DeleteFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::DeleteFile::Fn(self, request.into_inner()).await
	}

	async fn rename_file(&self, request:Request<RenameFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::RenameFile::Fn(self, request.into_inner()).await
	}

	async fn copy_file(&self, request:Request<CopyFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::CopyFile::Fn(self, request.into_inner()).await
	}

	async fn create_directory(&self, request:Request<CreateDirectoryRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::CreateDirectory::Fn(self, request.into_inner()).await
	}

	async fn open_document(
		&self,

		request:Request<OpenDocumentRequest>,
	) -> Result<Response<OpenDocumentResponse>, Status> {
		Workspace::OpenDocument::Fn(self, request.into_inner()).await
	}

	async fn save_all(&self, request:Request<SaveAllRequest>) -> Result<Response<SaveAllResponse>, Status> {
		Workspace::SaveAll::Fn(self, request.into_inner()).await
	}

	async fn apply_edit(&self, request:Request<ApplyEditRequest>) -> Result<Response<ApplyEditResponse>, Status> {
		Workspace::ApplyEdit::Fn(self, request.into_inner()).await
	}

	async fn update_configuration(
		&self,

		request:Request<UpdateConfigurationRequest>,
	) -> Result<Response<Empty>, Status> {
		Workspace::UpdateConfiguration::Fn(self, request.into_inner()).await
	}

	async fn update_workspace_folders(
		&self,

		request:Request<UpdateWorkspaceFoldersRequest>,
	) -> Result<Response<Empty>, Status> {
		Workspace::UpdateWorkspaceFolders::Fn(self, request.into_inner()).await
	}

	async fn open_terminal(&self, request:Request<OpenTerminalRequest>) -> Result<Response<Empty>, Status> {
		Terminal::OpenTerminal::Fn(self, request.into_inner()).await
	}

	async fn terminal_input(&self, request:Request<TerminalInputRequest>) -> Result<Response<Empty>, Status> {
		Terminal::TerminalInput::Fn(self, request.into_inner()).await
	}

	async fn close_terminal(&self, request:Request<CloseTerminalRequest>) -> Result<Response<Empty>, Status> {
		Terminal::CloseTerminal::Fn(self, request.into_inner()).await
	}

	async fn accept_terminal_opened(
		&self,

		request:Request<TerminalOpenedNotification>,
	) -> Result<Response<Empty>, Status> {
		Terminal::AcceptTerminalOpened::Fn(self, request.into_inner()).await
	}

	async fn accept_terminal_closed(
		&self,

		request:Request<TerminalClosedNotification>,
	) -> Result<Response<Empty>, Status> {
		Terminal::AcceptTerminalClosed::Fn(self, request.into_inner()).await
	}

	async fn accept_terminal_process_id(
		&self,

		request:Request<TerminalProcessIdNotification>,
	) -> Result<Response<Empty>, Status> {
		Terminal::AcceptTerminalProcessId::Fn(self, request.into_inner()).await
	}

	async fn accept_terminal_process_data(
		&self,

		request:Request<TerminalDataNotification>,
	) -> Result<Response<Empty>, Status> {
		Terminal::AcceptTerminalProcessData::Fn(self, request.into_inner()).await
	}

	async fn resize_terminal(&self, request:Request<ResizeTerminalRequest>) -> Result<Response<Empty>, Status> {
		Terminal::ResizeTerminal::Fn(self, request.into_inner()).await
	}

	async fn register_tree_view_provider(
		&self,

		request:Request<RegisterTreeViewProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		TreeView::RegisterTreeViewProvider::Fn(self, request.into_inner()).await
	}

	async fn get_tree_children(
		&self,

		request:Request<GetTreeChildrenRequest>,
	) -> Result<Response<GetTreeChildrenResponse>, Status> {
		TreeView::GetTreeChildren::Fn(self, request.into_inner()).await
	}

	async fn register_scm_provider(
		&self,

		request:Request<RegisterScmProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		SCM::RegisterScmProvider::Fn(self, request.into_inner()).await
	}

	async fn update_scm_group(&self, request:Request<UpdateScmGroupRequest>) -> Result<Response<Empty>, Status> {
		SCM::UpdateScmGroup::Fn(self, request.into_inner()).await
	}

	async fn git_exec(&self, request:Request<GitExecRequest>) -> Result<Response<GitExecResponse>, Status> {
		SCM::GitExec::Fn(self, request.into_inner()).await
	}

	async fn register_debug_adapter(
		&self,

		request:Request<RegisterDebugAdapterRequest>,
	) -> Result<Response<Empty>, Status> {
		Debug::RegisterDebugAdapter::Fn(self, request.into_inner()).await
	}

	async fn start_debugging(
		&self,

		request:Request<StartDebuggingRequest>,
	) -> Result<Response<StartDebuggingResponse>, Status> {
		Debug::StartDebugging::Fn(self, request.into_inner()).await
	}

	async fn stop_debugging(&self, request:Request<StopDebuggingRequest>) -> Result<Response<Empty>, Status> {
		Debug::StopDebugging::Fn(self, request.into_inner()).await
	}

	async fn participate_in_save(
		&self,

		request:Request<ParticipateInSaveRequest>,
	) -> Result<Response<ParticipateInSaveResponse>, Status> {
		Save::Fn(self, request.into_inner()).await
	}

	async fn get_secret(&self, request:Request<GetSecretRequest>) -> Result<Response<GetSecretResponse>, Status> {
		Secret::GetSecret::Fn(self, request.into_inner()).await
	}

	async fn store_secret(&self, request:Request<StoreSecretRequest>) -> Result<Response<Empty>, Status> {
		Secret::StoreSecret::Fn(self, request.into_inner()).await
	}

	async fn delete_secret(&self, request:Request<DeleteSecretRequest>) -> Result<Response<Empty>, Status> {
		Secret::DeleteSecret::Fn(self, request.into_inner()).await
	}

	async fn register_document_highlight_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterDocumentHighlightProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_document_highlights(
		&self,

		request:Request<ProvideDocumentHighlightsRequest>,
	) -> Result<Response<ProvideDocumentHighlightsResponse>, Status> {
		Provider::ProvideDocumentHighlights::Fn(self, request.into_inner()).await
	}

	async fn register_document_symbol_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterDocumentSymbolProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_document_symbols(
		&self,

		request:Request<ProvideDocumentSymbolsRequest>,
	) -> Result<Response<ProvideDocumentSymbolsResponse>, Status> {
		Provider::ProvideDocumentSymbols::Fn(self, request.into_inner()).await
	}

	async fn register_workspace_symbol_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterWorkspaceSymbolProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_workspace_symbols(
		&self,

		request:Request<ProvideWorkspaceSymbolsRequest>,
	) -> Result<Response<ProvideWorkspaceSymbolsResponse>, Status> {
		Provider::ProvideWorkspaceSymbols::Fn(self, request.into_inner()).await
	}

	async fn register_rename_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterRenameProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_rename_edits(
		&self,

		request:Request<ProvideRenameEditsRequest>,
	) -> Result<Response<ProvideRenameEditsResponse>, Status> {
		Provider::ProvideRenameEdits::Fn(self, request.into_inner()).await
	}

	async fn register_document_formatting_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterDocumentFormattingProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_document_formatting(
		&self,

		request:Request<ProvideDocumentFormattingRequest>,
	) -> Result<Response<ProvideDocumentFormattingResponse>, Status> {
		Provider::ProvideDocumentFormatting::Fn(self, request.into_inner()).await
	}

	async fn register_document_range_formatting_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterDocumentRangeFormattingProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_document_range_formatting(
		&self,

		request:Request<ProvideDocumentRangeFormattingRequest>,
	) -> Result<Response<ProvideDocumentRangeFormattingResponse>, Status> {
		Provider::ProvideDocumentRangeFormatting::Fn(self, request.into_inner()).await
	}

	async fn register_on_type_formatting_provider(
		&self,

		request:Request<RegisterOnTypeFormattingProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterOnTypeFormattingProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_on_type_formatting(
		&self,

		request:Request<ProvideOnTypeFormattingRequest>,
	) -> Result<Response<ProvideOnTypeFormattingResponse>, Status> {
		Provider::ProvideOnTypeFormatting::Fn(self, request.into_inner()).await
	}

	async fn register_signature_help_provider(
		&self,

		request:Request<RegisterSignatureHelpProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterSignatureHelpProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_signature_help(
		&self,

		request:Request<ProvideSignatureHelpRequest>,
	) -> Result<Response<ProvideSignatureHelpResponse>, Status> {
		Provider::ProvideSignatureHelp::Fn(self, request.into_inner()).await
	}

	async fn register_code_lens_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterCodeLensProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_code_lenses(
		&self,

		request:Request<ProvideCodeLensesRequest>,
	) -> Result<Response<ProvideCodeLensesResponse>, Status> {
		Provider::ProvideCodeLenses::Fn(self, request.into_inner()).await
	}

	async fn register_folding_range_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterFoldingRangeProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_folding_ranges(
		&self,

		request:Request<ProvideFoldingRangesRequest>,
	) -> Result<Response<ProvideFoldingRangesResponse>, Status> {
		Provider::ProvideFoldingRanges::Fn(self, request.into_inner()).await
	}

	async fn register_selection_range_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterSelectionRangeProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_selection_ranges(
		&self,

		request:Request<ProvideSelectionRangesRequest>,
	) -> Result<Response<ProvideSelectionRangesResponse>, Status> {
		Provider::ProvideSelectionRanges::Fn(self, request.into_inner()).await
	}

	async fn register_semantic_tokens_provider(
		&self,

		request:Request<RegisterSemanticTokensProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterSemanticTokensProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_semantic_tokens_full(
		&self,

		request:Request<ProvideSemanticTokensRequest>,
	) -> Result<Response<ProvideSemanticTokensResponse>, Status> {
		Provider::ProvideSemanticTokensFull::Fn(self, request.into_inner()).await
	}

	async fn register_inlay_hints_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterInlayHintsProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_inlay_hints(
		&self,

		request:Request<ProvideInlayHintsRequest>,
	) -> Result<Response<ProvideInlayHintsResponse>, Status> {
		Provider::ProvideInlayHints::Fn(self, request.into_inner()).await
	}

	async fn register_type_hierarchy_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterTypeHierarchyProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_type_hierarchy_supertypes(
		&self,

		request:Request<ProvideTypeHierarchyRequest>,
	) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
		Provider::ProvideTypeHierarchySupertypes::Fn(self, request.into_inner()).await
	}

	async fn provide_type_hierarchy_subtypes(
		&self,

		request:Request<ProvideTypeHierarchyRequest>,
	) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
		Provider::ProvideTypeHierarchySubtypes::Fn(self, request.into_inner()).await
	}

	async fn register_call_hierarchy_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterCallHierarchyProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_call_hierarchy_incoming_calls(
		&self,

		request:Request<ProvideCallHierarchyRequest>,
	) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
		Provider::ProvideCallHierarchyIncomingCalls::Fn(self, request.into_inner()).await
	}

	async fn provide_call_hierarchy_outgoing_calls(
		&self,

		request:Request<ProvideCallHierarchyRequest>,
	) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
		Provider::ProvideCallHierarchyOutgoingCalls::Fn(self, request.into_inner()).await
	}

	async fn register_linked_editing_range_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Provider::RegisterLinkedEditingRangeProvider::Fn(self, request.into_inner()).await
	}

	async fn provide_linked_editing_ranges(
		&self,

		request:Request<ProvideLinkedEditingRangesRequest>,
	) -> Result<Response<ProvideLinkedEditingRangesResponse>, Status> {
		Provider::ProvideLinkedEditingRanges::Fn(self, request.into_inner()).await
	}

	async fn register_inline_completion_item_provider(
		&self,

		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		// Registration: store handle + selector in the LanguageFeatureProviderRegistry.
		// The generic NotificationDispatcher already handles
		// `register_inline_completion_item_provider` coming via
		// SendCocoonNotification; this typed path is the proto-generated entry point.
		// NOTE: prost generates snake_case fields from the proto definition.
		let Inner = request.into_inner();

		let Handle = Inner.handle;

		let Selector = Inner.language_selector.clone();

		let ExtId = Inner.extension_id.clone();

		self.RegisterProvider(Handle, ProviderType::InlineCompletion, &Selector, &ExtId);

		Ok(Response::new(Empty {}))
	}

	async fn provide_inline_completion_items(
		&self,

		request:Request<ProvideInlineCompletionRequest>,
	) -> Result<Response<ProvideInlineCompletionResponse>, Status> {
		Provider::ProvideInlineCompletionItems::Fn(self, request.into_inner()).await
	}

	async fn show_quick_pick(
		&self,

		request:Request<ShowQuickPickRequest>,
	) -> Result<Response<ShowQuickPickResponse>, Status> {
		Window::ShowQuickPick::Fn(self, request.into_inner()).await
	}

	async fn show_input_box(
		&self,

		request:Request<ShowInputBoxRequest>,
	) -> Result<Response<ShowInputBoxResponse>, Status> {
		Window::ShowInputBox::Fn(self, request.into_inner()).await
	}

	async fn show_progress(
		&self,

		request:Request<ShowProgressRequest>,
	) -> Result<Response<ShowProgressResponse>, Status> {
		Window::ShowProgress::Fn(self, request.into_inner()).await
	}

	async fn report_progress(&self, request:Request<ReportProgressRequest>) -> Result<Response<Empty>, Status> {
		Window::ReportProgress::Fn(self, request.into_inner()).await
	}

	async fn open_external(&self, request:Request<OpenExternalRequest>) -> Result<Response<Empty>, Status> {
		Window::OpenExternal::Fn(self, request.into_inner()).await
	}

	async fn create_output_channel(
		&self,

		request:Request<CreateOutputChannelRequest>,
	) -> Result<Response<CreateOutputChannelResponse>, Status> {
		Output::CreateOutputChannel::Fn(self, request.into_inner()).await
	}

	async fn append_output(&self, request:Request<AppendOutputRequest>) -> Result<Response<Empty>, Status> {
		Output::AppendOutput::Fn(self, request.into_inner()).await
	}

	async fn clear_output(&self, request:Request<ClearOutputRequest>) -> Result<Response<Empty>, Status> {
		Output::ClearOutput::Fn(self, request.into_inner()).await
	}

	async fn show_output(&self, request:Request<ShowOutputRequest>) -> Result<Response<Empty>, Status> {
		Output::ShowOutput::Fn(self, request.into_inner()).await
	}

	async fn dispose_output(&self, request:Request<DisposeOutputRequest>) -> Result<Response<Empty>, Status> {
		Output::DisposeOutput::Fn(self, request.into_inner()).await
	}

	async fn register_task_provider(
		&self,

		request:Request<RegisterTaskProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Task::RegisterTaskProvider::Fn(self, request.into_inner()).await
	}

	async fn execute_task(&self, request:Request<ExecuteTaskRequest>) -> Result<Response<ExecuteTaskResponse>, Status> {
		Task::ExecuteTask::Fn(self, request.into_inner()).await
	}

	async fn terminate_task(&self, request:Request<TerminateTaskRequest>) -> Result<Response<Empty>, Status> {
		Task::TerminateTask::Fn(self, request.into_inner()).await
	}

	async fn get_authentication_session(
		&self,

		request:Request<GetAuthenticationSessionRequest>,
	) -> Result<Response<GetAuthenticationSessionResponse>, Status> {
		Auth::GetAuthenticationSession::Fn(self, request.into_inner()).await
	}

	async fn register_authentication_provider(
		&self,

		request:Request<RegisterAuthenticationProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		Auth::RegisterAuthenticationProvider::Fn(self, request.into_inner()).await
	}

	async fn get_extension(
		&self,

		request:Request<GetExtensionRequest>,
	) -> Result<Response<GetExtensionResponse>, Status> {
		Extension::GetExtension::Fn(self, request.into_inner()).await
	}

	async fn get_all_extensions(&self, request:Request<Empty>) -> Result<Response<GetAllExtensionsResponse>, Status> {
		Extension::GetAllExtensions::Fn(self, request.into_inner()).await
	}

	async fn get_configuration(
		&self,

		request:Request<GetConfigurationRequest>,
	) -> Result<Response<GetConfigurationResponse>, Status> {
		Extension::GetConfiguration::Fn(self, request.into_inner()).await
	}
}
