//! # CocoonServiceImpl Implementation
//!
//! This module implements the main gRPC service for Mountain-Cocoon

#[allow(unused_imports)]
// communication. It handles all requests from the Cocoon extension host
// sidecar.
//
// ## Service Responsibilities
//
// - **Initialization**: Handshake and extension host initialization
// - **Commands**: Register and execute extension commands
// - **Language Features**: Hover, completion, definition, references, code
// actions
// - **File System**: Read, write, stat, and watch files
// - **Terminal**: Manage terminal instances and I/O
// - **Tree View**: Register providers and get tree children
// - **SCM**: Source control management and git operations
// - **Debug**: Debug adapter registration and session management
// - **Save Participants**: Handle save events from extensions
//
// ## Architecture
//
// The service maintains references to:
// - `MountainEnvironment`: Access to all Mountain services and providers
// - `ActiveOperations`: Registry of cancellable operations
//
// ## Error Handling
//
// All methods return `tonic::Result<T>` and use proper error conversion
/// from internal errors to gRPC status codes.
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::Environment::MountainEnvironment::MountainEnvironment;
// Import generated protobuf types
use crate::Vine::Generated::{
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
	// Workspace Operations
	FindFilesRequest,
	FindFilesResponse,
	FindTextInFilesRequest,
	FindTextInFilesResponse,
	GenericNotification,
	// Common generic types
	GenericRequest,
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
	cocoon_service_server::CocoonService,
};

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
		info!("[CocoonService] New instance created");

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
		debug!("[CocoonService] Registered operation {} for cancellation", request_id);
		token
	}

	/// Unregisters an operation after completion
	///
	/// # Parameters
	/// - `request_id`: The request identifier to unregister
	pub async fn UnregisterOperation(&self, request_id:u64) {
		self.ActiveOperations.write().await.remove(&request_id);
		debug!("[CocoonService] Unregistered operation {}", request_id);
	}
}

#[async_trait]
impl CocoonService for CocoonServiceImpl {
	/// Process Mountain requests from Cocoon (generic request-response)
	async fn process_mountain_request(
		&self,
		request:Request<GenericRequest>,
	) -> Result<Response<GenericResponse>, Status> {
		let request_data = request.into_inner();
		info!(
			"[CocoonService] Processing generic Mountain request '{}' with ID {}",
			request_data.method, request_data.request_identifier
		);

		// Request router with method-to-handler mapping
		// This method provides a generic interface for all CocoonService operations
		// The actual implementation delegates to specific type-safe methods below
		warn!("[CocoonService] Generic request router not yet fully implemented - use type-safe methods instead");

		Ok(Response::new(GenericResponse {
			request_identifier:request_data.request_identifier,
			result:Vec::new(),
			error:Some(RpcError {
				code:-32601, // Method not found (JSON-RPC error code)
				message:format!("Method '{}' not implemented in generic router", request_data.method),
				data:Vec::new(),
			}),
		}))
	}

	/// Send Mountain notifications to Cocoon (generic fire-and-forget)
	async fn send_mountain_notification(
		&self,
		request:Request<GenericNotification>,
	) -> Result<Response<Empty>, Status> {
		let notification = request.into_inner();
		debug!(
			"[CocoonService] Sending generic Mountain notification '{}'",
			notification.method
		);

		// Notification router with method-to-handler mapping
		// This method provides a generic interface for fire-and-forget notifications
		// The actual implementation delegates to specific type-safe notification
		// methods
		debug!("[CocoonService] Generic notification router: method='{}'", notification.method);

		Ok(Response::new(Empty {}))
	}

	/// Cancel operations requested by Mountain
	async fn cancel_operation(&self, request:Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		let cancel_request = request.into_inner();
		info!(
			"[CocoonService] Cancel operation request: {}",
			cancel_request.request_identifier_to_cancel
		);

		// ActiveOperations tracking and cancellation logic
		if let Some(token) = self
			.ActiveOperations
			.read()
			.await
			.get(&cancel_request.request_identifier_to_cancel)
		{
			debug!(
				"[CocoonService] Triggering cancellation token for operation {}",
				cancel_request.request_identifier_to_cancel
			);
			token.cancel();
		} else {
			warn!(
				"[CocoonService] No active operation found for cancellation: {}",
				cancel_request.request_identifier_to_cancel
			);
		}

		Ok(Response::new(Empty {}))
	}

	// ==================== Initialization ====================

	/// Handshake - Called by Cocoon to signal readiness
	async fn initial_handshake(&self, _request:Request<Empty>) -> Result<Response<Empty>, Status> {
		info!("[CocoonService] Initial handshake received from Cocoon");
		Ok(Response::new(Empty {}))
	}

	/// Initialize Extension Host - Mountain sends initialization data to Cocoon
	async fn init_extension_host(&self, request:Request<InitExtensionHostRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Initializing extension host with {} workspace folders",
			req.workspace_folders.len()
		);

		// Initialize workspace folders in MountainEnvironment
		// This stub logs the workspace folders for debugging
		for folder in &req.workspace_folders {
			debug!(
				"[CocoonService] Workspace folder: {} ({})",
				folder.name,
				folder.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
			);
		}

		// Initialize configuration from request
		debug!("[CocoonService] Configuration: {} keys", req.configuration.len());

		// TODO: When ApplicationState is available:
		// - Store workspace folders in WorkspaceState
		// - Initialize configuration in ConfigurationState
		// - Notify registered extensions of initialization complete

		Ok(Response::new(Empty {}))
	}

	// ==================== Commands ====================

	/// Register Command - Cocoon registers an extension command
	async fn register_command(&self, request:Request<RegisterCommandRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Registering command '{}' from extension '{}'",
			req.command_id, req.extension_id
		);

		// Register command in MountainEnvironment
		// This stub logs the command registration for debugging
		debug!("[CocoonService] Command details: id={}, title={:?}", req.command_id, req.title);

		// TODO: When CommandRegistry is available in MountainEnvironment:
		// - Store command metadata in command registry
		// - Map command_id to extension handler
		// - Return success or error on duplicate registration

		Ok(Response::new(Empty {}))
	}

	/// Execute Contributed Command - Mountain executes an extension command
	async fn execute_contributed_command(
		&self,
		request:Request<ExecuteCommandRequest>,
	) -> Result<Response<ExecuteCommandResponse>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Executing command '{}' with {} arguments",
			req.command_id,
			req.arguments.len()
		);

		// Look up command handler and execute with parameters
		// This stub logs the command execution for debugging
		for (i, arg) in req.arguments.iter().enumerate() {
			debug!("[CocoonService] Argument {}: {:?}", i, arg);
		}

		// TODO: When CommandExecutor is available in MountainEnvironment:
		// - Look up command handler by command_id in registry
		// - Execute command with provided arguments
		// - Return success/error response with result data
		// - Handle command invocation errors gracefully

		// Return placeholder response until command execution is implemented
		Ok(Response::new(ExecuteCommandResponse {
			result:Some(crate::Vine::Generated::execute_command_response::Result::Value(
				b"Command execution not yet implemented".to_vec(),
			)),
		}))
	}

	/// Unregister Command - Unregister a previously registered command
	async fn unregister_command(&self, request:Request<UnregisterCommandRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Unregistering command '{}'", req.command_id);

		// Remove command from MountainEnvironment command registry
		// This stub logs the command unregistration for debugging
		debug!("[CocoonService] Removing command: {}", req.command_id);

		// TODO: When CommandRegistry is available in MountainEnvironment:
		// - Remove command from registry by command_id
		// - Clean up any associated command handlers
		// - Return success or warn if command not found

		Ok(Response::new(Empty {}))
	}

	// ==================== Language Features ====================

	/// Register Hover Provider - Register a hover provider
	async fn register_hover_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Registering hover provider for '{}' with handle {}",
			req.language_selector, req.handle
		);

		// Store provider in MountainEnvironment provider registry
		debug!(
			"[CocoonService] Hover provider registered: handle={}, language={}",
			req.handle, req.language_selector
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector, trigger_characters)
		// - Map handle to extension for RPC callbacks
		// - Return error on duplicate handle registration

		Ok(Response::new(Empty {}))
	}

	/// Provide Hover - Request hover information
	async fn provide_hover(
		&self,
		request:Request<ProvideHoverRequest>,
	) -> Result<Response<ProvideHoverResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing hover for provider {}", req.provider_handle);

		// Look up provider by handle in MountainEnvironment, call provider via RPC,
		// return hover result This stub returns an empty hover response
		warn!(
			"[CocoonService] Provider lookup not yet implemented - handle: {}",
			req.provider_handle
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by provider_handle
		// - Call extension backend via gRPC with position triggers
		// - Parse hover markdown and range from response
		// - Return formatted hover response

		Ok(Response::new(ProvideHoverResponse {
			markdown:"Hover provider not yet implemented".to_string(),
			range:Some(Range {
				start:Some(Position { line:0, character:0 }),
				end:Some(Position { line:0, character:0 }),
			}),
		}))
	}

	/// Register Completion Item Provider - Register a completion provider
	async fn register_completion_item_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Registering completion provider for '{}' with handle {}",
			req.language_selector, req.handle
		);

		// Store provider in MountainEnvironment provider registry
		debug!(
			"[CocoonService] Completion provider registered: handle={}, language={}",
			req.handle, req.language_selector
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector, trigger_chars)
		// - Map handle to extension for RPC callbacks
		// - Register trigger characters

		Ok(Response::new(Empty {}))
	}

	/// Provide Completion Items - Request completion items
	async fn provide_completion_items(
		&self,
		request:Request<ProvideCompletionItemsRequest>,
	) -> Result<Response<ProvideCompletionItemsResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing completions for provider {}", req.provider_handle);

		// Look up provider by handle in MountainEnvironment
		// This stub returns an empty completion list
		warn!(
			"[CocoonService] Provider lookup not yet implemented - handle: {}",
			req.provider_handle
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by provider_handle
		// - Call extension backend via gRPC with position context
		// - Parse completion items from response
		// - Return formatted completion items list

		Ok(Response::new(ProvideCompletionItemsResponse { items:Vec::new() }))
	}

	/// Register Definition Provider - Register a definition provider
	async fn register_definition_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Registering definition provider for '{}' with handle {}",
			req.language_selector, req.handle
		);

		// Store provider in MountainEnvironment provider registry
		debug!(
			"[CocoonService] Definition provider registered: handle={}, language={}",
			req.handle, req.language_selector
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// Provide Definition - Request definition location
	async fn provide_definition(
		&self,
		request:Request<ProvideDefinitionRequest>,
	) -> Result<Response<ProvideDefinitionResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing definition for provider {}", req.provider_handle);

		// Look up provider by handle in MountainEnvironment
		// This stub returns an empty location list
		warn!(
			"[CocoonService] Provider lookup not yet implemented - handle: {}",
			req.provider_handle
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by provider_handle
		// - Call extension backend via gRPC with position context
		// - Parse location(s) from response
		// - Return formatted locations list

		Ok(Response::new(ProvideDefinitionResponse { locations:Vec::new() }))
	}

	/// Register Reference Provider - Register a reference provider
	async fn register_reference_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Registering reference provider for '{}' with handle {}",
			req.language_selector, req.handle
		);

		// Store provider in MountainEnvironment provider registry
		debug!(
			"[CocoonService] Reference provider registered: handle={}, language={}",
			req.handle, req.language_selector
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// Provide References - Request references
	async fn provide_references(
		&self,
		request:Request<ProvideReferencesRequest>,
	) -> Result<Response<ProvideReferencesResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing references for provider {}", req.provider_handle);

		// Look up provider by handle in MountainEnvironment
		// This stub returns an empty location list
		warn!(
			"[CocoonService] Provider lookup not yet implemented - handle: {}",
			req.provider_handle
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by provider_handle
		// - Call extension backend via gRPC with position context
		// - Parse location(s) from response
		// - Return formatted locations list

		Ok(Response::new(ProvideReferencesResponse { locations:Vec::new() }))
	}

	/// Register Code Actions Provider - Register code actions provider
	async fn register_code_actions_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Registering code actions provider for '{}' with handle {}",
			req.language_selector, req.handle
		);

		// Store provider in MountainEnvironment provider registry
		debug!(
			"[CocoonService] Code actions provider registered: handle={}, language={}",
			req.handle, req.language_selector
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector, action_kinds)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// Provide Code Actions - Request code actions
	async fn provide_code_actions(
		&self,
		request:Request<ProvideCodeActionsRequest>,
	) -> Result<Response<ProvideCodeActionsResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing code actions for provider {}", req.provider_handle);

		// Look up provider by handle in MountainEnvironment
		// This stub returns an empty actions list
		warn!(
			"[CocoonService] Provider lookup not yet implemented - handle: {}",
			req.provider_handle
		);

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by provider_handle
		// - Call extension backend via gRPC with range and diagnostics context
		// - Parse code actions from response
		// - Return formatted code actions list

		Ok(Response::new(ProvideCodeActionsResponse { actions:Vec::new() }))
	}

	// ==================== Window Operations ====================

	/// Show Text Document - Open a text document
	async fn show_text_document(
		&self,
		request:Request<ShowTextDocumentRequest>,
	) -> Result<Response<ShowTextDocumentResponse>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Showing text document: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// IPC call to Wind frontend to open document tab
		// This stub logs the request for debugging
		debug!(
			"[CocoonService] Would send IPC to Wind: open_document uri={:?} column={:?}",
			req.uri.map(|u| u.value),
			req.view_column
		);

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send message to Wind frontend via IPC
		// - Include URI, view column, and selection options
		// - Wait for acknowledgment or handle errors
		// - Return success/failure status

		Ok(Response::new(ShowTextDocumentResponse { success:true }))
	}

	/// Show Information Message - Display an info message
	async fn show_information_message(
		&self,
		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Showing information message");

		// IPC call to Wind frontend for message display
		info!("{}", req.message);
		// TODO: ShowMessageRequest only has 'message' field (no actions)

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send message to Wind frontend via IPC
		// - Include message type, text, and action buttons
		// - Wait for user action selection or dismissal
		// - Return selected action index
		warn!("{}", req.message);

		Ok(Response::new(ShowMessageResponse { success:true }))
	}

	/// Show Warning Message - Display a warning message
	async fn show_warning_message(
		&self,
		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Showing warning message");

		// IPC call to Wind frontend for message display
		info!("{}", req.message);
		// TODO: ShowMessageRequest only has 'message' field (no actions)

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send message to Wind frontend via IPC
		// - Include message type, text, and action buttons
		// - Wait for user action selection or dismissal
		// - Return selected action index
		warn!("{}", req.message);

		Ok(Response::new(ShowMessageResponse { success:true }))
	}

	/// Show Error Message - Display an error message
	async fn show_error_message(
		&self,
		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Showing error message");

		// IPC call to Wind frontend for message display
		info!("{}", req.message);
		// TODO: ShowMessageRequest only has 'message' field (no actions)

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send message to Wind frontend via IPC
		// - Include message type, text, and action buttons
		// - Wait for user action selection or dismissal
		// - Return selected action index
		error!("{}", req.message);

		Ok(Response::new(ShowMessageResponse { success:true }))
	}

	/// Create Status Bar Item - Create a status bar item
	async fn create_status_bar_item(
		&self,
		request:Request<CreateStatusBarItemRequest>,
	) -> Result<Response<CreateStatusBarItemResponse>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Creating status bar item: {}", req.id);

		// IPC call to Wind frontend for status bar item creation
		debug!("[CocoonService] Status bar item details: id={}, text={:?}", req.id, req.text);
		// Note: CreateStatusBarItemRequest has fields: id, text, tooltip (no alignment)

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send creation request to Wind frontend via IPC
		// - Include item ID, text, alignment, command, and priority
		// - Return item_id for future updates

		Ok(Response::new(CreateStatusBarItemResponse { item_id:req.id.clone() }))
	}

	/// Set Status Bar Text - Set status bar text
	async fn set_status_bar_text(&self, request:Request<SetStatusBarTextRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Setting status bar text for item {}", req.item_id);

		// IPC call to Wind frontend for status bar update
		debug!("[CocoonService] Update details: item_id={}, text={}", req.item_id, req.text);
		// Note: SetStatusBarTextRequest has fields: item_id, text (no tooltip)

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send update request to Wind frontend via IPC
		// - Include item_id, text, tooltip, and color
		// - Handle errors for invalid item_id

		Ok(Response::new(Empty {}))
	}

	/// Create Webview Panel - Create a new webview panel
	async fn create_webview_panel(
		&self,
		request:Request<CreateWebviewPanelRequest>,
	) -> Result<Response<CreateWebviewPanelResponse>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Creating webview panel: {}", req.view_type);

		// IPC call to Wind frontend to create webview panel
		// This stub returns a placeholder handle (0)
		debug!(
			"[CocoonService] Panel details: view_type={}, title={}",
			req.view_type, req.title
		);
		// Note: CreateWebviewPanelRequest fields: view_type, title, icon_path,
		// view_column, preserve_focus, etc. (no options)

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Generate unique handle for the panel
		// - Send creation request to Wind via IPC
		// - Include view_type, title, options, and content
		// - Return handle to caller
		// - Generate unique handle
		// - Send creation request to Wind
		// - Return handle to caller

		Ok(Response::new(CreateWebviewPanelResponse { handle:0 }))
	}

	/// Set Webview HTML - Update webview HTML content
	async fn set_webview_html(&self, request:Request<SetWebviewHtmlRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Setting webview HTML for handle {}", req.handle);

		// IPC call to Wind frontend for HTML update
		debug!("[CocoonService] HTML length: {} bytes", req.html.len());

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send HTML update request to Wind via IPC
		// - Include handle and HTML content
		// - Handle errors for invalid handle

		Ok(Response::new(Empty {}))
	}

	/// On Did Receive Message - Receive message from webview
	async fn on_did_receive_message(
		&self,
		request:Request<OnDidReceiveMessageRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Received webview message for handle {}", req.handle);

		// Forward to extension handler registered in MountainEnvironment
		debug!(
			"[CocoonService] Message payload: {}",
			req.message.as_ref().map_or("absent", |_| "present")
		);

		// TODO: When WebviewHandlerRegistry is available in MountainEnvironment:
		// - Look up handler by handle
		// - Forward message to registered extension
		// - Handle errors for invalid handle or missing handler

		Ok(Response::new(Empty {}))
	}

	// ==================== File System ====================

	/// Read File - Read file contents
	async fn read_file(&self, request:Request<ReadFileRequest>) -> Result<Response<ReadFileResponse>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Reading file: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// Delegate to FileSystemProvider in MountainEnvironment
		// This stub returns an unimplemented error
		warn!("[CocoonService] FileSystemProvider not yet available in MountainEnvironment");

		// TODO: When FileSystemProvider is available in MountainEnvironment:
		// - Parse URI to filesystem path
		// - Call FileSystemProvider method
		// - Return result or error
		// - Handle errors (file not found, permission denied, etc.)

		Err(Status::unimplemented("read_file not yet implemented"))
	}

	/// Write File - Write file contents
	async fn write_file(&self, request:Request<WriteFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Writing file: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// Delegate to FileSystemProvider in MountainEnvironment
		// This stub returns an unimplemented error
		warn!("[CocoonService] FileSystemProvider not yet available in MountainEnvironment");

		// TODO: When FileSystemProvider is available in MountainEnvironment:
		// - Parse URI to filesystem path
		// - Call FileSystemProvider method
		// - Return result or error
		// - Handle errors (file not found, permission denied, etc.)

		Err(Status::unimplemented("write_file not yet implemented"))
	}

	/// Stat - Get file metadata
	async fn stat(&self, request:Request<StatRequest>) -> Result<Response<StatResponse>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Getting file metadata: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// Delegate to FileSystemProvider in MountainEnvironment
		// This stub returns an unimplemented error
		warn!("[CocoonService] FileSystemProvider not yet available in MountainEnvironment");

		// TODO: When FileSystemProvider is available in MountainEnvironment:
		// - Parse URI to filesystem path
		// - Call FileSystemProvider method
		// - Return result or error
		// - Handle errors (file not found, permission denied, etc.)

		Err(Status::unimplemented("stat not yet implemented"))
	}

	/// Read Directory - List directory contents
	async fn readdir(&self, request:Request<ReaddirRequest>) -> Result<Response<ReaddirResponse>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Reading directory: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// Delegate to FileSystemProvider in MountainEnvironment
		// This stub returns an unimplemented error
		warn!("[CocoonService] FileSystemProvider not yet available in MountainEnvironment");

		// TODO: When FileSystemProvider is available in MountainEnvironment:
		// - Parse URI to filesystem path
		// - Call FileSystemProvider method
		// - Return result or error
		// - Handle errors (file not found, permission denied, etc.)

		Err(Status::unimplemented("readdir not yet implemented"))
	}

	/// Watch File - Watch file for changes
	async fn watch_file(&self, request:Request<WatchFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Watching file: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// Delegate to FileSystemProvider with file watcher integration in
		// MountainEnvironment This stub returns an unimplemented error
		// Note: WatchFileRequest only has 'uri' field (no options)

		// TODO: When FileSystemProvider is available in MountainEnvironment:
		// - Parse URI to filesystem path
		// - Register watcher with FileSystemProvider
		// - Store watcher ID in ApplicationState for cancellation
		// - Handle errors (not found, permission denied, too many watchers)

		Err(Status::unimplemented("watch_file not yet implemented"))
	}

	// ==================== Workspace Operations ====================

	/// Find Files - Search for files
	async fn find_files(&self, request:Request<FindFilesRequest>) -> Result<Response<FindFilesResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Finding files with pattern: {}", req.pattern);

		// Delegate to FileSystemProvider or SearchProvider in MountainEnvironment
		// This stub returns an unimplemented error
		debug!(
			"[CocoonService] Search details: pattern={}, include={:?}",
			req.pattern, req.include
		);

		// TODO: When SearchProvider is available in MountainEnvironment:
		// - Use SearchProvider.find_files with glob patterns
		// - Return matching URIs (FindFilesResponse has no max_results limit)

		Err(Status::unimplemented("find_files not yet implemented"))
	}

	/// Find Text in Files - Search for text across files
	async fn find_text_in_files(
		&self,
		request:Request<FindTextInFilesRequest>,
	) -> Result<Response<FindTextInFilesResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Finding text with pattern: {}", req.pattern);

		// Delegate to SearchProvider with text search index in MountainEnvironment
		// This stub returns an unimplemented error
		debug!(
			"[CocoonService] Search details: pattern={}, include={:?}",
			req.pattern, req.include
		);

		// TODO: When SearchProvider is available in MountainEnvironment:
		// - Use SearchProvider.find_text_in_files with full-text index
		// - Return matches with line/column context

		Err(Status::unimplemented("find_text_in_files not yet implemented"))
	}

	/// Open Document - Open a document
	async fn open_document(
		&self,
		request:Request<OpenDocumentRequest>,
	) -> Result<Response<OpenDocumentResponse>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Opening document: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// IPC call to Wind frontend to open document tab
		// This stub logs the request for debugging
		debug!(
			"[CocoonService] Would send IPC to Wind: open_document uri={:?} column={:?}",
			req.uri.map(|u| u.value),
			req.view_column
		);

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send message to Wind frontend via IPC
		// - Include URI, view column, and selection options
		// - Wait for acknowledgment or handle errors
		// - Return success/failure status

		Err(Status::unimplemented("open_document not yet implemented"))
	}

	/// Save All - Save all open documents
	async fn save_all(&self, request:Request<SaveAllRequest>) -> Result<Response<SaveAllResponse>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Saving all documents (includeUntitled: {})",
			req.include_untitled
		);

		// IPC call to Wind frontend to save all documents
		// This stub returns an unimplemented error
		warn!("[CocoonService] Save all not yet implemented");

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send save_all request to Wind via IPC
		// - Include includeUntitled flag
		// - Wait for completion
		// - Return success/failure and saved document count

		Err(Status::unimplemented("save_all not yet implemented"))
	}

	/// Apply Edit - Apply a text edit to a document
	async fn apply_edit(&self, request:Request<ApplyEditRequest>) -> Result<Response<ApplyEditResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Applying {} edits to document", req.edits.len());

		// IPC call to Wind frontend to apply edits
		// This stub returns an unimplemented error
		debug!("[CocoonService] Edit target: {:?}", req.uri.map(|u| u.value));

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Send apply_edit request to Wind via IPC
		// - Include URI, edits, and document version
		// - Wait for apply completion
		// - Return result with applied edits and rejected edits

		Err(Status::unimplemented("apply_edit not yet implemented"))
	}

	/// Update Configuration - Notify of configuration changes
	async fn update_configuration(
		&self,
		request:Request<UpdateConfigurationRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Updating configuration with {} changed keys",
			req.changed_keys.len()
		);

		// Update ConfigurationState in MountainEnvironment
		for key in &req.changed_keys {
			debug!("[CocoonService] Configuration key changed: {}", key);
		}

		// TODO: When ConfigurationState is available in MountainEnvironment:
		// - Update configuration in ConfigurationState
		// - Notify registered configuration change listeners
		// - Handle errors for invalid keys

		Ok(Response::new(Empty {}))
	}

	/// Update Workspace Folders - Update workspace folders
	async fn update_workspace_folders(
		&self,
		request:Request<UpdateWorkspaceFoldersRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Updating workspace: {} additions, {} removals",
			req.additions.len(),
			req.removals.len()
		);

		// Update WorkspaceState in MountainEnvironment
		for addition in &req.additions {
			debug!(
				"[CocoonService] Adding workspace folder: {} ({})",
				addition.name,
				addition.uri.as_ref().map(|u| &u.value).unwrap_or(&"?".to_string())
			);
		}
		for removal in &req.removals {
			debug!(
				"[CocoonService] Removing workspace folder: {}",
				removal.uri.as_ref().map(|u| &u.value).unwrap_or(&"?".to_string())
			);
		}

		// TODO: When WorkspaceState is available in MountainEnvironment:
		// - Update workspace folders in WorkspaceState
		// - Notify registered workspace listeners
		// - Handle errors for duplicate additions or missing removals

		Ok(Response::new(Empty {}))
	}

	// ==================== Terminal ====================

	/// Open Terminal - Open a new terminal
	async fn open_terminal(&self, request:Request<OpenTerminalRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Opening terminal: {}", req.name);

		// IPC call to Wind frontend and TerminalProvider in MountainEnvironment
		// This stub returns an unimplemented error
		debug!(
			"[CocoonService] Terminal options: cwd={:?}, shell_path={:?}",
			req.cwd, req.shell_path
		);

		// TODO: When TerminalProvider is available in MountainEnvironment:
		// - Create PTY using TerminalProvider
		// - Send terminal creation request to Wind via IPC
		// - Return terminal_id for future operations
		// - Handle errors (too many terminals, shell not found, etc.)

		Err(Status::unimplemented("open_terminal not yet implemented"))
	}

	/// Terminal Input - Send input to terminal
	async fn terminal_input(&self, request:Request<TerminalInputRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Sending input to terminal {}", req.terminal_id);

		// Forward to TerminalProvider for PTY input in MountainEnvironment
		// This stub returns an unimplemented error
		debug!("[CocoonService] Input length: {} bytes", req.data.len());

		// TODO: When TerminalProvider is available in MountainEnvironment:
		// - Look up PTY by terminal_id
		// - Write input bytes to PTY
		// - Handle errors (terminal not found, PTY closed)
		// - Forward to Wind via IPC for display updates

		Err(Status::unimplemented("terminal_input not yet implemented"))
	}

	/// Close Terminal - Close a terminal
	async fn close_terminal(&self, request:Request<CloseTerminalRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Closing terminal {}", req.terminal_id);

		// Close PTY and notify Wind frontend in MountainEnvironment
		// This stub returns an unimplemented error
		warn!("[CocoonService] Terminal close not yet implemented");

		// TODO: When TerminalProvider is available in MountainEnvironment:
		// - Look up PTY by terminal_id
		// - Close PTY and cleanup resources
		// - Notify Wind via IPC
		// - Remove from TerminalState
		// - Handle errors (terminal not found)

		Err(Status::unimplemented("close_terminal not yet implemented"))
	}

	/// Accept Terminal Opened - Notification: Terminal opened
	async fn accept_terminal_opened(
		&self,
		request:Request<TerminalOpenedNotification>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Terminal opened notification: {} (ID: {})",
			req.name, req.terminal_id
		);

		// Forward to terminal event handlers registered in MountainEnvironment
		debug!("[CocoonService] Terminal event notification received");

		// TODO: When TerminalState is available in MountainEnvironment:
		// - Update or remove terminal from TerminalState
		// - Notify registered terminal event handlers
		// - Forward to Wind via IPC for UI updates

		Ok(Response::new(Empty {}))
	}

	/// Accept Terminal Closed - Notification: Terminal closed
	async fn accept_terminal_closed(
		&self,
		request:Request<TerminalClosedNotification>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Terminal closed notification: {}", req.terminal_id);

		// Forward to terminal event handlers registered in MountainEnvironment
		debug!("[CocoonService] Terminal event notification received");

		// TODO: When TerminalState is available in MountainEnvironment:
		// - Update or remove terminal from TerminalState
		// - Notify registered terminal event handlers
		// - Forward to Wind via IPC for UI updates

		Ok(Response::new(Empty {}))
	}

	/// Accept Terminal Process ID - Notification: Terminal process ID
	async fn accept_terminal_process_id(
		&self,
		request:Request<TerminalProcessIdNotification>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Terminal process ID: {} for terminal {}",
			req.process_id, req.terminal_id
		);

		// Store in TerminalState in MountainEnvironment
		debug!("[CocoonService] Process ID received for terminal {}", req.terminal_id);

		// TODO: When TerminalState is available in MountainEnvironment:
		// - Update terminal metadata with process ID
		// - Notify registered terminal event handlers
		// - Store for future reference (e.g., process killing)

		Ok(Response::new(Empty {}))
	}

	/// Accept Terminal Process Data - Notification: Terminal output
	async fn accept_terminal_process_data(
		&self,
		request:Request<TerminalDataNotification>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Terminal data for {}: {} bytes",
			req.terminal_id,
			req.data.len()
		);

		// Forward to Wind frontend via IPC in MountainEnvironment
		// This stub logs the data for debugging
		trace!("[CocoonService] Terminal output: {}", String::from_utf8_lossy(&req.data));

		// TODO: When Wind IPC layer is available in MountainEnvironment:
		// - Forward output data to Wind via IPC
		// - Include terminal_id and data bytes
		// - Handle encoding (UTF-8 validation)
		// - Buffer data if IPC channel is congested

		Ok(Response::new(Empty {}))
	}

	// ==================== Tree View ====================

	/// Register Tree View Provider - Register a tree view provider
	async fn register_tree_view_provider(
		&self,
		request:Request<RegisterTreeViewProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering tree view provider: {}", req.view_id);

		// Store provider in MountainEnvironment TreeViewState
		debug!(
			"[CocoonService] Tree view provider registered: view_id={}, extension_id={:?}",
			req.view_id, req.extension_id
		);
		// Note: RegisterTreeViewProviderRequest fields: view_id, extension_id (no
		// display_name)

		// TODO: When TreeViewState is available in MountainEnvironment:
		// - Store provider metadata in TreeViewState
		// - Map view_id to extension for RPC callbacks
		// - Store provide_root_item flag and initial root if provided
		// - Register with Wind for UI display

		Ok(Response::new(Empty {}))
	}

	/// Get Tree Children - Request tree view children
	async fn get_tree_children(
		&self,
		request:Request<GetTreeChildrenRequest>,
	) -> Result<Response<GetTreeChildrenResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Getting tree children for view {}", req.view_id);

		// Look up provider and call GetTreeChildren in MountainEnvironment
		// This stub returns an empty list
		warn!(
			"[CocoonService] Tree view provider lookup not yet implemented - view_id: {}",
			req.view_id
		);

		// TODO: When TreeViewState is available in MountainEnvironment:
		// - Look up provider by view_id
		// - Call extension backend via gRPC with element_handle and parent_handle
		// - Parse tree items from response
		// - Cache items in TreeViewState for performance
		// - Return formatted tree items list

		Ok(Response::new(GetTreeChildrenResponse { items:Vec::new() }))
	}

	// ==================== SCM ====================

	/// Register SCM Provider - Register source control provider
	async fn register_scm_provider(
		&self,
		request:Request<RegisterScmProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering SCM provider: {}", req.scm_id);

		// Store SCM provider in MountainEnvironment
		debug!(
			"[CocoonService] SCM provider registered: scm_id={}, extension_id={:?}",
			req.scm_id, req.extension_id
		);
		// Note: RegisterScmProviderRequest fields: scm_id, extension_id (no
		// display_name)

		// TODO: When SCMState is available in MountainEnvironment:
		// - Store provider metadata in SCMState
		// - Map scm_id to extension for RPC callbacks
		// - Register with Wind for UI display
		// - Store supported commands and features

		Ok(Response::new(Empty {}))
	}

	/// Update SCM Group - Update SCM group
	async fn update_scm_group(&self, request:Request<UpdateScmGroupRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Updating SCM group {} with provider {}",
			req.group_id, req.provider_id
		);

		// Update SCM group in MountainEnvironment
		debug!(
			"[CocoonService] Group update details: provider_id={}, group_id={}, resource_states={:?}",
			req.provider_id, req.group_id, req.resource_states
		);
		// Note: UpdateScmGroupRequest fields: provider_id, group_id, resource_states
		// (no label, state)

		// TODO: When SCMState is available in MountainEnvironment:
		// - Update group metadata in SCMState
		// - Update resource states if provided
		// - Notify Wind of changes via IPC
		// - Handle errors for invalid group_id or provider_id

		Ok(Response::new(Empty {}))
	}

	/// Execute Git - Execute git command
	async fn git_exec(&self, request:Request<GitExecRequest>) -> Result<Response<GitExecResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Executing git command: {}", req.args.join(" "));

		// Delegate to SCM provider for git execution in MountainEnvironment
		// This stub returns an unimplemented error
		debug!(
			"[CocoonService] Git execution details: repository_path={:?}, args={:?}",
			req.repository_path, req.args
		);
		// Note: GitExecRequest fields: repository_path (string), args (repeated string)

		// TODO: When SCMProvider is available in MountainEnvironment:
		// - Look up SCM provider for the repository
		// - Execute git command via git2 crate or spawn git process
		// - Capture stdout, stderr, and exit code
		// - Return formatted response with results
		// - Handle errors (git not found, repository corruption, etc.)

		Err(Status::unimplemented("git_exec requires SCMProvider in MountainEnvironment"))
	}

	// ==================== Debug ====================

	/// Register Debug Adapter - Register debug adapter
	async fn register_debug_adapter(
		&self,
		request:Request<RegisterDebugAdapterRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering debug adapter: {}", req.debug_type);

		// Register debug adapter in MountainEnvironment DebugState
		debug!(
			"[CocoonService] Debug adapter registered: debug_type={}, extension_id={:?}",
			req.debug_type, req.extension_id
		);

		// TODO: When DebugState is available in MountainEnvironment:
		// - Store adapter metadata in DebugState
		// - Map debug_type to adapter executable path
		// - Store supported DAP features
		// - Register with Wind for UI display

		Ok(Response::new(Empty {}))
	}

	/// Start Debugging - Start debug session
	async fn start_debugging(
		&self,
		request:Request<StartDebuggingRequest>,
	) -> Result<Response<StartDebuggingResponse>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Starting debugging session: {}", req.debug_type);

		// Spawn debug adapter and start DAP session in MountainEnvironment
		// This stub returns failure
		warn!(
			"[CocoonService] Debug session start not yet implemented - type: {}",
			req.debug_type
		);
		debug!("[CocoonService] Debug configuration: {:?}", req.configuration);

		// TODO: When DebugAdapterExecutor is available in MountainEnvironment:
		// - Look up debug adapter by debug_type
		// - Spawn debug adapter process
		// - Initialize DAP session with configuration
		// - Handle DAP protocol messages
		// - Return success/failure status
		// - Attach breakpoints and watch expressions

		Ok(Response::new(StartDebuggingResponse { success:false }))
	}

	// ==================== Save Participants ====================

	/// Participate in Save - Extension participates in save
	async fn participate_in_save(
		&self,
		request:Request<ParticipateInSaveRequest>,
	) -> Result<Response<ParticipateInSaveResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Participating in save for: {:?}", req.uri);

		// Look up registered save participants and aggregate edits in
		// MountainEnvironment
		debug!("[CocoonService] Save reason: {:?}", req.reason);

		// TODO: When SaveParticipantRegistry is available in MountainEnvironment:
		// - Look up save participants for this URI
		// - Call each participant with document context and reason
		// - Aggregate TextEditForSave responses from all participants
		// - Return consolidated edits list
		// - Handle errors from individual participants

		Ok(Response::new(ParticipateInSaveResponse { edits:Vec::new() }))
	}

	// ==================== Secret Storage ====================

	/// Get Secret - Retrieve a secret from storage
	async fn get_secret(&self, request:Request<GetSecretRequest>) -> Result<Response<GetSecretResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Getting secret for key: {}", req.key);

		// Delegate to SecretStorageProvider in MountainEnvironment
		// This stub returns an unimplemented error
		warn!("[CocoonService] SecretStorageProvider not yet available in MountainEnvironment");

		// TODO: When SecretStorageProvider is available in MountainEnvironment:
		// - Look up secret by key in SecretStorageProvider
		// - Return secret value or error if not found
		// - Handle encryption/decryption
		// - Handle permission errors

		Err(Status::unimplemented(
			"get_secret requires SecretStorageProvider in MountainEnvironment",
		))
	}

	/// Store Secret - Store a secret in storage
	async fn store_secret(&self, request:Request<StoreSecretRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Storing secret for key: {}", req.key);

		// Delegate to SecretStorageProvider in MountainEnvironment
		// This stub returns an unimplemented error
		debug!("[CocoonService] Secret value length: {} bytes", req.value.len());

		// TODO: When SecretStorageProvider is available in MountainEnvironment:
		// - Store secret with key in SecretStorageProvider
		// - Handle encryption before storage
		// - Update existing secret or create new one
		// - Handle permission errors and storage limits

		Err(Status::unimplemented(
			"store_secret requires SecretStorageProvider in MountainEnvironment",
		))
	}

	/// Delete Secret - Delete a secret from storage
	async fn delete_secret(&self, request:Request<DeleteSecretRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Deleting secret for key: {}", req.key);

		// Delegate to SecretStorageProvider in MountainEnvironment
		// This stub returns an unimplemented error
		warn!("[CocoonService] Secret deletion not yet implemented");

		// TODO: When SecretStorageProvider is available in MountainEnvironment:
		// - Remove secret by key from SecretStorageProvider
		// - Return success or error if not found
		// - Handle permission errors

		Err(Status::unimplemented(
			"delete_secret requires SecretStorageProvider in MountainEnvironment",
		))
	}

	// ==================== Extended Language Provider Handlers ====================

	/// Document Highlight Provider - Register
	async fn register_document_highlight_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Document Highlight Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// document highlights - Provide
	async fn provide_document_highlights(
		&self,
		request:Request<ProvideDocumentHighlightsRequest>,
	) -> Result<Response<ProvideDocumentHighlightsResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing document highlights");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideDocumentHighlightsResponse::default()))
	}

	/// Document Symbol Provider - Register
	async fn register_document_symbol_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Document Symbol Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// document symbols - Provide
	async fn provide_document_symbols(
		&self,
		request:Request<ProvideDocumentSymbolsRequest>,
	) -> Result<Response<ProvideDocumentSymbolsResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing document symbols");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideDocumentSymbolsResponse::default()))
	}

	/// Workspace Symbol Provider - Register
	async fn register_workspace_symbol_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Workspace Symbol Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// workspace symbols - Provide
	async fn provide_workspace_symbols(
		&self,
		request:Request<ProvideWorkspaceSymbolsRequest>,
	) -> Result<Response<ProvideWorkspaceSymbolsResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing workspace symbols");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideWorkspaceSymbolsResponse::default()))
	}

	/// Rename Provider - Register
	async fn register_rename_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Rename Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// rename edits - Provide
	async fn provide_rename_edits(
		&self,
		request:Request<ProvideRenameEditsRequest>,
	) -> Result<Response<ProvideRenameEditsResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing rename edits");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideRenameEditsResponse::default()))
	}

	/// Document Formatting Provider - Register
	async fn register_document_formatting_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Document Formatting Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// document formatting - Provide
	async fn provide_document_formatting(
		&self,
		request:Request<ProvideDocumentFormattingRequest>,
	) -> Result<Response<ProvideDocumentFormattingResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing document formatting");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideDocumentFormattingResponse::default()))
	}

	/// Document Range Formatting Provider - Register
	async fn register_document_range_formatting_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Document Range Formatting Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// document range formatting - Provide
	async fn provide_document_range_formatting(
		&self,
		request:Request<ProvideDocumentRangeFormattingRequest>,
	) -> Result<Response<ProvideDocumentRangeFormattingResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing document range formatting");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideDocumentRangeFormattingResponse::default()))
	}

	/// On Type Formatting Provider - Register
	async fn register_on_type_formatting_provider(
		&self,
		request:Request<RegisterOnTypeFormattingProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering On Type Formatting Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// on-type formatting - Provide
	async fn provide_on_type_formatting(
		&self,
		request:Request<ProvideOnTypeFormattingRequest>,
	) -> Result<Response<ProvideOnTypeFormattingResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing on-type formatting");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideOnTypeFormattingResponse::default()))
	}

	/// Signature Help Provider - Register
	async fn register_signature_help_provider(
		&self,
		request:Request<RegisterSignatureHelpProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Signature Help Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// signature help - Provide
	async fn provide_signature_help(
		&self,
		request:Request<ProvideSignatureHelpRequest>,
	) -> Result<Response<ProvideSignatureHelpResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing signature help");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideSignatureHelpResponse::default()))
	}

	/// Code Lens Provider - Register
	async fn register_code_lens_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Code Lens Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// code lenses - Provide
	async fn provide_code_lenses(
		&self,
		request:Request<ProvideCodeLensesRequest>,
	) -> Result<Response<ProvideCodeLensesResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing code lenses");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideCodeLensesResponse::default()))
	}

	/// Folding Range Provider - Register
	async fn register_folding_range_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Folding Range Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// folding ranges - Provide
	async fn provide_folding_ranges(
		&self,
		request:Request<ProvideFoldingRangesRequest>,
	) -> Result<Response<ProvideFoldingRangesResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing folding ranges");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideFoldingRangesResponse::default()))
	}

	/// Selection Range Provider - Register
	async fn register_selection_range_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Selection Range Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// selection ranges - Provide
	async fn provide_selection_ranges(
		&self,
		request:Request<ProvideSelectionRangesRequest>,
	) -> Result<Response<ProvideSelectionRangesResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing selection ranges");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideSelectionRangesResponse::default()))
	}

	/// Semantic Tokens Provider - Register
	async fn register_semantic_tokens_provider(
		&self,
		request:Request<RegisterSemanticTokensProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Semantic Tokens Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// semantic tokens - Provide
	async fn provide_semantic_tokens_full(
		&self,
		request:Request<ProvideSemanticTokensRequest>,
	) -> Result<Response<ProvideSemanticTokensResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing semantic tokens");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideSemanticTokensResponse::default()))
	}

	/// Inlay Hints Provider - Register
	async fn register_inlay_hints_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Inlay Hints Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// inlay hints - Provide
	async fn provide_inlay_hints(
		&self,
		request:Request<ProvideInlayHintsRequest>,
	) -> Result<Response<ProvideInlayHintsResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing inlay hints");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideInlayHintsResponse::default()))
	}

	/// Type Hierarchy Provider - Register
	async fn register_type_hierarchy_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Type Hierarchy Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// type hierarchy supertypes - Provide
	async fn provide_type_hierarchy_supertypes(
		&self,
		request:Request<ProvideTypeHierarchyRequest>,
	) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing type hierarchy supertypes");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideTypeHierarchyResponse::default()))
	}

	/// type hierarchy subtypes - Provide
	async fn provide_type_hierarchy_subtypes(
		&self,
		request:Request<ProvideTypeHierarchyRequest>,
	) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing type hierarchy subtypes");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideTypeHierarchyResponse::default()))
	}

	/// Call Hierarchy Provider - Register
	async fn register_call_hierarchy_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Call Hierarchy Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// call hierarchy incoming - Provide
	async fn provide_call_hierarchy_incoming_calls(
		&self,
		request:Request<ProvideCallHierarchyRequest>,
	) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing call hierarchy incoming");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideCallHierarchyResponse::default()))
	}

	/// call hierarchy outgoing - Provide
	async fn provide_call_hierarchy_outgoing_calls(
		&self,
		request:Request<ProvideCallHierarchyRequest>,
	) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing call hierarchy outgoing");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideCallHierarchyResponse::default()))
	}

	/// Linked Editing Range Provider - Register
	async fn register_linked_editing_range_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Linked Editing Range Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// linked editing ranges - Provide
	async fn provide_linked_editing_ranges(
		&self,
		request:Request<ProvideLinkedEditingRangesRequest>,
	) -> Result<Response<ProvideLinkedEditingRangesResponse>, Status> {
		let _req = request.into_inner();
		debug!("[CocoonService] Providing linked editing ranges");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Look up provider by handle
		// - Call extension backend via gRPC
		// - Return result

		Ok(Response::new(ProvideLinkedEditingRangesResponse::default()))
	}

	/// quick pick
	async fn show_quick_pick(
		&self,
		request:Request<ShowQuickPickRequest>,
	) -> Result<Response<ShowQuickPickResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling quick pick");

		// TODO: Implement quick pick in MountainEnvironment

		Ok(Response::new(ShowQuickPickResponse { ..Default::default() }))
	}

	/// input box
	async fn show_input_box(
		&self,
		request:Request<ShowInputBoxRequest>,
	) -> Result<Response<ShowInputBoxResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling input box");

		// TODO: Implement input box in MountainEnvironment

		Ok(Response::new(ShowInputBoxResponse { ..Default::default() }))
	}

	/// progress
	async fn show_progress(
		&self,
		request:Request<ShowProgressRequest>,
	) -> Result<Response<ShowProgressResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling progress");

		// TODO: Implement progress in MountainEnvironment

		Ok(Response::new(ShowProgressResponse { ..Default::default() }))
	}

	/// progress report
	async fn report_progress(&self, request:Request<ReportProgressRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling progress report");

		// TODO: Implement progress report in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// webview message
	async fn post_webview_message(
		&self,
		request:Request<PostWebviewMessageRequest>,
	) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling webview message");

		// TODO: Implement webview message in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// webview dispose
	async fn dispose_webview_panel(
		&self,
		request:Request<DisposeWebviewPanelRequest>,
	) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling webview dispose");

		// TODO: Implement webview dispose in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// external URI
	async fn open_external(&self, request:Request<OpenExternalRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling external URI");

		// TODO: Implement external URI in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// file delete
	async fn delete_file(&self, request:Request<DeleteFileRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling file delete");

		// TODO: Implement file delete in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// file rename
	async fn rename_file(&self, request:Request<RenameFileRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling file rename");

		// TODO: Implement file rename in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// file copy
	async fn copy_file(&self, request:Request<CopyFileRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling file copy");

		// TODO: Implement file copy in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// directory creation
	async fn create_directory(&self, request:Request<CreateDirectoryRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling directory creation");

		// TODO: Implement directory creation in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// output channel creation
	async fn create_output_channel(
		&self,
		request:Request<CreateOutputChannelRequest>,
	) -> Result<Response<CreateOutputChannelResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling output channel creation");

		// TODO: Implement output channel creation in MountainEnvironment

		Ok(Response::new(CreateOutputChannelResponse { ..Default::default() }))
	}

	/// output append
	async fn append_output(&self, request:Request<AppendOutputRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling output append");

		// TODO: Implement output append in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// output clear
	async fn clear_output(&self, request:Request<ClearOutputRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling output clear");

		// TODO: Implement output clear in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// output show
	async fn show_output(&self, request:Request<ShowOutputRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling output show");

		// TODO: Implement output show in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// output dispose
	async fn dispose_output(&self, request:Request<DisposeOutputRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling output dispose");

		// TODO: Implement output dispose in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// Task Provider - Register
	async fn register_task_provider(
		&self,
		request:Request<RegisterTaskProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Task Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// task execution
	async fn execute_task(&self, request:Request<ExecuteTaskRequest>) -> Result<Response<ExecuteTaskResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling task execution");

		// TODO: Implement task execution in MountainEnvironment

		Ok(Response::new(ExecuteTaskResponse { ..Default::default() }))
	}

	/// task termination
	async fn terminate_task(&self, request:Request<TerminateTaskRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling task termination");

		// TODO: Implement task termination in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// authentication session
	async fn get_authentication_session(
		&self,
		request:Request<GetAuthenticationSessionRequest>,
	) -> Result<Response<GetAuthenticationSessionResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling authentication session");

		// TODO: Implement authentication session in MountainEnvironment

		Ok(Response::new(GetAuthenticationSessionResponse { ..Default::default() }))
	}

	/// Authentication Provider - Register
	async fn register_authentication_provider(
		&self,
		request:Request<RegisterAuthenticationProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Authentication Provider");

		// TODO: When ProviderRegistry is available in MountainEnvironment:
		// - Store provider metadata (handle, language_selector)
		// - Map handle to extension for RPC callbacks

		Ok(Response::new(Empty {}))
	}

	/// debug stop
	async fn stop_debugging(&self, request:Request<StopDebuggingRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling debug stop");

		// TODO: Implement debug stop in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// extension info
	async fn get_extension(
		&self,
		request:Request<GetExtensionRequest>,
	) -> Result<Response<GetExtensionResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling extension info");

		// TODO: Implement extension info in MountainEnvironment

		Ok(Response::new(GetExtensionResponse { ..Default::default() }))
	}

	/// all extensions
	async fn get_all_extensions(&self, request:Request<Empty>) -> Result<Response<GetAllExtensionsResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling all extensions");

		// TODO: Implement all extensions in MountainEnvironment

		Ok(Response::new(GetAllExtensionsResponse { ..Default::default() }))
	}

	/// terminal resize
	async fn resize_terminal(&self, request:Request<ResizeTerminalRequest>) -> Result<Response<Empty>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling terminal resize");

		// TODO: Implement terminal resize in MountainEnvironment

		Ok(Response::new(Empty {}))
	}

	/// configuration value
	async fn get_configuration(
		&self,
		request:Request<GetConfigurationRequest>,
	) -> Result<Response<GetConfigurationResponse>, Status> {
		let _req = request.into_inner();
		info!("[CocoonService] Handling configuration value");

		// TODO: Implement configuration value in MountainEnvironment

		Ok(Response::new(GetConfigurationResponse { ..Default::default() }))
	}
}
