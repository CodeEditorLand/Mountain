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
use log::{debug, error, info, warn};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::Environment::MountainEnvironment::MountainEnvironment;
// Import generated protobuf types
use crate::Vine::Generated::{
	ApplyEditRequest,
	ApplyEditResponse,
	Argument,
	CancelOperationRequest,

	CloseTerminalRequest,
	CodeAction,

	CompletionItem,
	CreateStatusBarItemRequest,
	CreateStatusBarItemResponse,
	CreateWebviewPanelRequest,
	CreateWebviewPanelResponse,
	DebugConfiguration,
	DeleteSecretRequest,
	// Common types
	Empty,
	ExecuteCommandRequest,
	ExecuteCommandResponse,
	// Workspace Operations
	FindFilesRequest,
	FindFilesResponse,
	FindTextInFilesRequest,
	FindTextInFilesResponse,
	GenericNotification,
	// Common generic types
	GenericRequest,
	GenericResponse,
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
	// Terminal
	OpenTerminalRequest,
	// Save Participants
	ParticipateInSaveRequest,
	ParticipateInSaveResponse,
	Position,
	ProvideCodeActionsRequest,
	ProvideCodeActionsResponse,
	ProvideCompletionItemsRequest,
	ProvideCompletionItemsResponse,
	ProvideDefinitionRequest,
	ProvideDefinitionResponse,
	ProvideHoverRequest,
	ProvideHoverResponse,
	ProvideReferencesRequest,
	ProvideReferencesResponse,
	Range,
	// File System
	ReadFileRequest,
	ReadFileResponse,
	ReaddirRequest,
	ReaddirResponse,
	// Commands
	RegisterCommandRequest,
	// Debug
	RegisterDebugAdapterRequest,
	// Language Features
	RegisterProviderRequest,
	// SCM
	RegisterScmProviderRequest,
	// Tree View
	RegisterTreeViewProviderRequest,
	SaveAllRequest,
	SaveAllResponse,
	SetStatusBarTextRequest,
	SetWebviewHtmlRequest,
	ShowMessageRequest,
	ShowMessageResponse,
	// Window Operations
	ShowTextDocumentRequest,
	ShowTextDocumentResponse,
	SourceControlResourceState,
	StartDebuggingRequest,
	StartDebuggingResponse,

	StatRequest,
	StatResponse,
	StoreSecretRequest,
	TerminalClosedNotification,
	TerminalDataNotification,

	TerminalInputRequest,
	TerminalOpenedNotification,
	TerminalProcessIdNotification,
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
	// Service trait
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

		// TODO: Implement generic request handling
		// - Route to appropriate handler based on method
		// - Return response or error

		Ok(Response::new(GenericResponse {
			request_identifier:request_data.request_identifier,
			result:Vec::new(),
			error:None,
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

		// TODO: Implement generic notification handling
		// - Route to appropriate handler based on method
		// - Return success

		Ok(Response::new(Empty {}))
	}

	/// Cancel operations requested by Mountain
	async fn cancel_operation(&self, request:Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		let cancel_request = request.into_inner();
		info!(
			"[CocoonService] Cancel operation request: {}",
			cancel_request.request_identifier_to_cancel
		);

		// TODO: Implement operation cancellation
		// - Look up operation in ActiveOperations
		// - Trigger cancellation token
		// - Return success

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

		// TODO: Implement proper initialization logic
		// - Store workspace folders
		// - Initialize configuration
		// - Notify extensions of initialization

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

		// TODO: Implement command registration in the command registry
		// - Store command metadata
		// - Register with CommandExecutor

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

		// TODO: Implement command execution
		// - Look up command in registry
		// - Execute with provided arguments
		// - Return result or error

		// For now, return a placeholder response
		Ok(Response::new(ExecuteCommandResponse {
			result:Some(crate::Vine::Generated::execute_command_response::Result::Value(
				b"placeholder".to_vec(),
			)),
		}))
	}

	/// Unregister Command - Unregister a previously registered command
	async fn unregister_command(&self, request:Request<UnregisterCommandRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Unregistering command '{}'", req.command_id);

		// TODO: Implement command unregistration

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

		// TODO: Implement hover provider registration

		Ok(Response::new(Empty {}))
	}

	/// Provide Hover - Request hover information
	async fn provide_hover(
		&self,
		request:Request<ProvideHoverRequest>,
	) -> Result<Response<ProvideHoverResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing hover for provider {}", req.provider_handle);

		// TODO: Implement hover provider lookup and execution
		// - Find provider by handle
		// - Call provider with position
		// - Return hover information

		Ok(Response::new(ProvideHoverResponse {
			markdown:String::new(),
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

		// TODO: Implement completion provider registration

		Ok(Response::new(Empty {}))
	}

	/// Provide Completion Items - Request completion items
	async fn provide_completion_items(
		&self,
		request:Request<ProvideCompletionItemsRequest>,
	) -> Result<Response<ProvideCompletionItemsResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing completions for provider {}", req.provider_handle);

		// TODO: Implement completion provider lookup and execution

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

		// TODO: Implement definition provider registration

		Ok(Response::new(Empty {}))
	}

	/// Provide Definition - Request definition location
	async fn provide_definition(
		&self,
		request:Request<ProvideDefinitionRequest>,
	) -> Result<Response<ProvideDefinitionResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing definition for provider {}", req.provider_handle);

		// TODO: Implement definition provider lookup and execution

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

		// TODO: Implement reference provider registration

		Ok(Response::new(Empty {}))
	}

	/// Provide References - Request references
	async fn provide_references(
		&self,
		request:Request<ProvideReferencesRequest>,
	) -> Result<Response<ProvideReferencesResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing references for provider {}", req.provider_handle);

		// TODO: Implement reference provider lookup and execution

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

		// TODO: Implement code actions provider registration

		Ok(Response::new(Empty {}))
	}

	/// Provide Code Actions - Request code actions
	async fn provide_code_actions(
		&self,
		request:Request<ProvideCodeActionsRequest>,
	) -> Result<Response<ProvideCodeActionsResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Providing code actions for provider {}", req.provider_handle);

		// TODO: Implement code actions provider lookup and execution

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

		// TODO: Implement document opening via IPC to Wind

		Ok(Response::new(ShowTextDocumentResponse { success:true }))
	}

	/// Show Information Message - Display an info message
	async fn show_information_message(
		&self,
		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Showing information message");

		// TODO: Implement via IPC to Wind
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

		// TODO: Implement via IPC to Wind
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

		// TODO: Implement via IPC to Wind
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

		// TODO: Implement status bar item creation via IPC to Wind

		Ok(Response::new(CreateStatusBarItemResponse { item_id:req.id.clone() }))
	}

	/// Set Status Bar Text - Set status bar text
	async fn set_status_bar_text(&self, request:Request<SetStatusBarTextRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Setting status bar text for item {}", req.item_id);

		// TODO: Implement via IPC to Wind

		Ok(Response::new(Empty {}))
	}

	/// Create Webview Panel - Create a new webview panel
	async fn create_webview_panel(
		&self,
		request:Request<CreateWebviewPanelRequest>,
	) -> Result<Response<CreateWebviewPanelResponse>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Creating webview panel: {}", req.view_type);

		// TODO: Implement webview panel creation via IPC to Wind
		// - Generate unique handle
		// - Send creation request to Wind
		// - Return handle to caller

		Ok(Response::new(CreateWebviewPanelResponse { handle:0 }))
	}

	/// Set Webview HTML - Update webview HTML content
	async fn set_webview_html(&self, request:Request<SetWebviewHtmlRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Setting webview HTML for handle {}", req.handle);

		// TODO: Implement via IPC to Wind

		Ok(Response::new(Empty {}))
	}

	/// On Did Receive Message - Receive message from webview
	async fn on_did_receive_message(
		&self,
		request:Request<OnDidReceiveMessageRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Received webview message for handle {}", req.handle);

		// TODO: Forward message to appropriate extension handler

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

		// TODO: Implement file reading via FileSystem provider

		Err(Status::unimplemented("read_file not yet implemented"))
	}

	/// Write File - Write file contents
	async fn write_file(&self, request:Request<WriteFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Writing file: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// TODO: Implement file writing via FileSystem provider

		Err(Status::unimplemented("write_file not yet implemented"))
	}

	/// Stat - Get file metadata
	async fn stat(&self, request:Request<StatRequest>) -> Result<Response<StatResponse>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Getting file metadata: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// TODO: Implement file stat via FileSystem provider

		Err(Status::unimplemented("stat not yet implemented"))
	}

	/// Read Directory - List directory contents
	async fn readdir(&self, request:Request<ReaddirRequest>) -> Result<Response<ReaddirResponse>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Reading directory: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// TODO: Implement directory reading via FileSystem provider

		Err(Status::unimplemented("readdir not yet implemented"))
	}

	/// Watch File - Watch file for changes
	async fn watch_file(&self, request:Request<WatchFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Watching file: {}",
			req.uri.as_ref().map(|u| &u.value).unwrap_or(&String::new())
		);

		// TODO: Implement file watching via FileSystem provider

		Err(Status::unimplemented("watch_file not yet implemented"))
	}

	// ==================== Workspace Operations ====================

	/// Find Files - Search for files
	async fn find_files(&self, request:Request<FindFilesRequest>) -> Result<Response<FindFilesResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Finding files with pattern: {}", req.pattern);

		// TODO: Implement file search via FileSystem or Search provider

		Err(Status::unimplemented("find_files not yet implemented"))
	}

	/// Find Text in Files - Search for text across files
	async fn find_text_in_files(
		&self,
		request:Request<FindTextInFilesRequest>,
	) -> Result<Response<FindTextInFilesResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Finding text with pattern: {}", req.pattern);

		// TODO: Implement text search via Search provider

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

		// TODO: Implement document opening via IPC to Wind

		Err(Status::unimplemented("open_document not yet implemented"))
	}

	/// Save All - Save all open documents
	async fn save_all(&self, request:Request<SaveAllRequest>) -> Result<Response<SaveAllResponse>, Status> {
		let req = request.into_inner();
		info!(
			"[CocoonService] Saving all documents (includeUntitled: {})",
			req.include_untitled
		);

		// TODO: Implement save all via IPC to Wind

		Err(Status::unimplemented("save_all not yet implemented"))
	}

	/// Apply Edit - Apply a text edit to a document
	async fn apply_edit(&self, request:Request<ApplyEditRequest>) -> Result<Response<ApplyEditResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Applying {} edits to document", req.edits.len());

		// TODO: Implement edit application via IPC to Wind

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

		// TODO: Implement configuration update

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

		// TODO: Implement workspace folder update

		Ok(Response::new(Empty {}))
	}

	// ==================== Terminal ====================

	/// Open Terminal - Open a new terminal
	async fn open_terminal(&self, request:Request<OpenTerminalRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Opening terminal: {}", req.name);

		// TODO: Implement terminal opening via IPC to Wind

		Err(Status::unimplemented("open_terminal not yet implemented"))
	}

	/// Terminal Input - Send input to terminal
	async fn terminal_input(&self, request:Request<TerminalInputRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Sending input to terminal {}", req.terminal_id);

		// TODO: Implement terminal input via IPC to Wind

		Err(Status::unimplemented("terminal_input not yet implemented"))
	}

	/// Close Terminal - Close a terminal
	async fn close_terminal(&self, request:Request<CloseTerminalRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Closing terminal {}", req.terminal_id);

		// TODO: Implement terminal closing via IPC to Wind

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

		// TODO: Forward notification to appropriate handlers

		Ok(Response::new(Empty {}))
	}

	/// Accept Terminal Closed - Notification: Terminal closed
	async fn accept_terminal_closed(
		&self,
		request:Request<TerminalClosedNotification>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Terminal closed notification: {}", req.terminal_id);

		// TODO: Forward notification to appropriate handlers

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

		// TODO: Store process ID for terminal

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

		// TODO: Forward terminal output to appropriate handlers

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

		// TODO: Implement tree view provider registration

		Ok(Response::new(Empty {}))
	}

	/// Get Tree Children - Request tree view children
	async fn get_tree_children(
		&self,
		request:Request<GetTreeChildrenRequest>,
	) -> Result<Response<GetTreeChildrenResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Getting tree children for view {}", req.view_id);

		// TODO: Implement tree children retrieval

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

		// TODO: Implement SCM provider registration

		Ok(Response::new(Empty {}))
	}

	/// Update SCM Group - Update SCM group
	async fn update_scm_group(&self, request:Request<UpdateScmGroupRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!(
			"[CocoonService] Updating SCM group {} with provider {}",
			req.group_id, req.provider_id
		);

		// TODO: Implement SCM group update

		Ok(Response::new(Empty {}))
	}

	/// Execute Git - Execute git command
	async fn git_exec(&self, request:Request<GitExecRequest>) -> Result<Response<GitExecResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Executing git command: {}", req.args.join(" "));

		// TODO: Implement git execution via SCM provider

		Err(Status::unimplemented("git_exec not yet implemented"))
	}

	// ==================== Debug ====================

	/// Register Debug Adapter - Register debug adapter
	async fn register_debug_adapter(
		&self,
		request:Request<RegisterDebugAdapterRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering debug adapter: {}", req.debug_type);

		// TODO: Implement debug adapter registration

		Ok(Response::new(Empty {}))
	}

	/// Start Debugging - Start debug session
	async fn start_debugging(
		&self,
		request:Request<StartDebuggingRequest>,
	) -> Result<Response<StartDebuggingResponse>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Starting debugging session: {}", req.debug_type);

		// TODO: Implement debugging session start

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

		// TODO: Implement save participant logic
		// - Call all registered save participants
		// - Collect text edits
		// - Return aggregated edits

		Ok(Response::new(ParticipateInSaveResponse { edits:Vec::new() }))
	}

	// ==================== Secret Storage ====================

	/// Get Secret - Retrieve a secret from storage
	async fn get_secret(&self, request:Request<GetSecretRequest>) -> Result<Response<GetSecretResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Getting secret for key: {}", req.key);

		// TODO: Implement secret retrieval via SecretStorage provider

		Err(Status::unimplemented("get_secret not yet implemented"))
	}

	/// Store Secret - Store a secret in storage
	async fn store_secret(&self, request:Request<StoreSecretRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Storing secret for key: {}", req.key);

		// TODO: Implement secret storage via SecretStorage provider

		Err(Status::unimplemented("store_secret not yet implemented"))
	}

	/// Delete Secret - Delete a secret from storage
	async fn delete_secret(&self, request:Request<DeleteSecretRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Deleting secret for key: {}", req.key);

		// TODO: Implement secret deletion via SecretStorage provider

		Err(Status::unimplemented("delete_secret not yet implemented"))
	}
}
