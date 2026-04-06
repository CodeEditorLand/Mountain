// # CocoonServiceImpl Implementation
//
// This module implements the main gRPC service for Mountain-Cocoon
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
// from internal errors to gRPC status codes.

#[allow(unused_imports)]
use std::{
	collections::HashMap,
	sync::Arc,
	time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	LanguageFeature::DTO::ProviderType::ProviderType,
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
use log::{debug, error, info, trace, warn};
use serde_json::json;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::{
	ApplicationState::DTO::{
		ProviderRegistrationDTO::ProviderRegistrationDTO,
		WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	},
	Environment::MountainEnvironment::MountainEnvironment,
};
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
	ExtensionInfo,
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
		let dto = ProviderRegistrationDTO {
			Handle:handle,
			ProviderType:provider_type,
			Selector:json!({ "language": [language_selector] }),
			SideCarIdentifier:extension_id.to_string(),
			ExtensionIdentifier:json!(extension_id),
			Options:None,
		};
		self.environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.RegisterProvider(handle, dto);
		debug!(
			"[CocoonService] Provider {:?} registered: handle={}, language={}",
			provider_type, handle, language_selector
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
	/// Process Mountain requests from Cocoon (generic request-response).
	///
	/// Routes legacy `fs.*` / `commands.*` / `secrets.*` method names used by
	/// Cocoon's `FileSystemService` and other services that call Mountain via
	/// the generic `ProcessCocoonRequest` RPC instead of the typed methods.
	///
	/// Parameters are JSON-encoded bytes in `request.parameter`. Results are
	/// JSON-encoded bytes in `response.result`.
	async fn process_mountain_request(
		&self,
		request:Request<GenericRequest>,
	) -> Result<Response<GenericResponse>, Status> {
		let Req = request.into_inner();
		let RequestId = Req.request_identifier;

		debug!("[CocoonService] generic request: method={} id={}", Req.method, RequestId);

		/// Serialise a value into the `result` bytes of a GenericResponse.
		fn OkResponse(RequestId:u64, Value:&impl serde::Serialize) -> Response<GenericResponse> {
			let Bytes = serde_json::to_vec(Value).unwrap_or_default();
			Response::new(GenericResponse { request_identifier:RequestId, result:Bytes, error:None })
		}

		/// Build an error GenericResponse.
		fn ErrResponse(RequestId:u64, Code:i32, Message:String) -> Response<GenericResponse> {
			Response::new(GenericResponse {
				request_identifier:RequestId,
				result:Vec::new(),
				error:Some(RpcError { code:Code, message:Message, data:Vec::new() }),
			})
		}

		// Deserialise the generic parameter bytes as a JSON value
		let Params:serde_json::Value = if Req.parameter.is_empty() {
			serde_json::Value::Null
		} else {
			serde_json::from_slice(&Req.parameter).unwrap_or(serde_json::Value::Null)
		};

		match Req.method.as_str() {
			// ---- File System ---- (Cocoon FileSystemService uses these paths)
			"fs.readFile" | "file:read" => {
				let Path = Params
					.as_str()
					.or_else(|| Params.get("path").and_then(|V| V.as_str()))
					.unwrap_or("");
				match tokio::fs::read(Path).await {
					Ok(Content) => Ok(OkResponse(RequestId, &Content)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.readFile: {}", Error))),
				}
			},
			"fs.writeFile" | "file:write" => {
				let Path = Params.get("path").and_then(|V| V.as_str()).unwrap_or("");
				let Content:Vec<u8> = Params
					.get("content")
					.and_then(|V| V.as_array())
					.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
					.unwrap_or_default();
				match tokio::fs::write(Path, &Content).await {
					Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.writeFile: {}", Error))),
				}
			},
			"fs.stat" | "file:stat" => {
				let Path = Params
					.as_str()
					.or_else(|| Params.get("path").and_then(|V| V.as_str()))
					.unwrap_or("");
				match tokio::fs::metadata(Path).await {
					Ok(Meta) => {
						let Mtime = Meta
							.modified()
							.ok()
							.and_then(|T| T.duration_since(UNIX_EPOCH).ok())
							.map(|D| D.as_millis() as u64)
							.unwrap_or(0);
						Ok(OkResponse(
							RequestId,
							&json!({
								"type": if Meta.is_dir() { 2 } else { 1 },
								"is_file": Meta.is_file(),
								"is_directory": Meta.is_dir(),
								"size": Meta.len(),
								"mtime": Mtime,
							}),
						))
					},
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.stat: {}", Error))),
				}
			},
			"fs.listDir" | "fs.readdir" | "file:readdir" => {
				let Path = Params
					.as_str()
					.or_else(|| Params.get("path").and_then(|V| V.as_str()))
					.unwrap_or("");
				match tokio::fs::read_dir(Path).await {
					Ok(mut Entries) => {
						// Return [{name, type}] where type 1=File 2=Directory
						let mut Items:Vec<serde_json::Value> = Vec::new();
						while let Ok(Some(Entry)) = Entries.next_entry().await {
							if let Some(Name) = Entry.file_name().to_str() {
								let IsDir = Entry.file_type().await.map(|T| T.is_dir()).unwrap_or(false);
								Items.push(json!({ "name": Name, "type": if IsDir { 2u32 } else { 1u32 } }));
							}
						}
						Ok(OkResponse(RequestId, &Items))
					},
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.listDir: {}", Error))),
				}
			},
			"fs.createDir" | "file:mkdir" => {
				let Path = Params
					.as_str()
					.or_else(|| Params.get("path").and_then(|V| V.as_str()))
					.unwrap_or("");
				match tokio::fs::create_dir_all(Path).await {
					Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.createDir: {}", Error))),
				}
			},
			"fs.delete" | "file:delete" => {
				let Path = Params
					.as_str()
					.or_else(|| Params.get("path").and_then(|V| V.as_str()))
					.unwrap_or("");
				let Result = if std::path::Path::new(Path).is_dir() {
					tokio::fs::remove_dir_all(Path).await
				} else {
					tokio::fs::remove_file(Path).await
				};
				match Result {
					Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.delete: {}", Error))),
				}
			},
			"fs.rename" | "file:move" => {
				let From = Params.get("from").and_then(|V| V.as_str()).unwrap_or("");
				let To = Params.get("to").and_then(|V| V.as_str()).unwrap_or("");
				match tokio::fs::rename(From, To).await {
					Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.rename: {}", Error))),
				}
			},
			// ---- Commands ----
			"commands.execute" => {
				let CommandId = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Arg = Params.get("arg").cloned().unwrap_or(serde_json::Value::Null);
				match self.environment.ExecuteCommand(CommandId, Arg).await {
					Ok(Value) => Ok(OkResponse(RequestId, &Value)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			// ---- Commands (Cocoon MountainGRPCClient format) ----
			"executeCommand" => {
				let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Arg = Params
					.get("arguments")
					.and_then(|A| A.as_array())
					.and_then(|A| A.first())
					.cloned()
					.unwrap_or(serde_json::Value::Null);
				match self.environment.ExecuteCommand(CommandId, Arg).await {
					Ok(Value) => Ok(OkResponse(RequestId, &json!({ "result": Value }))),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			"unregisterCommand" => {
				let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				match self.environment.UnregisterCommand(ExtensionId, CommandId).await {
					Ok(()) => Ok(OkResponse(RequestId, &json!({ "success": true }))),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			// ---- Window dialogs (Window.ts method names) ----
			"UserInterface.ShowOpenDialog" => {
				use CommonLibrary::UserInterface::{
					DTO::OpenDialogOptionsDTO::OpenDialogOptionsDTO,
					UserInterfaceProvider::UserInterfaceProvider,
				};
				let Title = Params
					.get(0)
					.and_then(|V| V.get("title"))
					.and_then(|T| T.as_str())
					.map(|S| S.to_string());
				let Options = OpenDialogOptionsDTO { Base: CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO { Title, ..Default::default() }, ..OpenDialogOptionsDTO::default() };
				match self.environment.ShowOpenDialog(Some(Options)).await {
					Ok(Some(Paths)) => {
						let Uris:Vec<String> = Paths.iter().map(|P| format!("file://{}", P.display())).collect();
						Ok(OkResponse(RequestId, &json!(Uris)))
					},
					Ok(None) => Ok(OkResponse(RequestId, &json!(serde_json::Value::Array(vec![])))),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			"UserInterface.ShowSaveDialog" => {
				use CommonLibrary::UserInterface::{
					DTO::SaveDialogOptionsDTO::SaveDialogOptionsDTO,
					UserInterfaceProvider::UserInterfaceProvider,
				};
				let Title = Params
					.get(0)
					.and_then(|V| V.get("title"))
					.and_then(|T| T.as_str())
					.map(|S| S.to_string());
				let Options = SaveDialogOptionsDTO { Base: CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO { Title, ..Default::default() }, ..SaveDialogOptionsDTO::default() };
				match self.environment.ShowSaveDialog(Some(Options)).await {
					Ok(Some(Path)) => Ok(OkResponse(RequestId, &json!(format!("file://{}", Path.display())))),
					Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			"UserInterface.ShowInputBox" => {
				use CommonLibrary::UserInterface::{
					DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
					UserInterfaceProvider::UserInterfaceProvider,
				};
				let Opts = Params.get(0);
				let Options = InputBoxOptionsDTO {
					Prompt:Opts
						.and_then(|V| V.get("prompt"))
						.and_then(|P| P.as_str())
						.map(|S| S.to_string()),
					PlaceHolder:Opts
						.and_then(|V| V.get("placeHolder"))
						.and_then(|P| P.as_str())
						.map(|S| S.to_string()),
					IsPassword:Some(Opts.and_then(|V| V.get("password")).and_then(|B| B.as_bool()).unwrap_or(false)),
					Value:Opts
						.and_then(|V| V.get("value"))
						.and_then(|V| V.as_str())
						.map(|S| S.to_string()),
					Title:None,
					IgnoreFocusOut:None,
				};
				match self.environment.ShowInputBox(Some(Options)).await {
					Ok(Some(Text)) => Ok(OkResponse(RequestId, &json!(Text))),
					Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			// ---- Native shell operations ----
			"openExternal" => {
				use tauri::Emitter;
				let Url = Params
					.as_str()
					.or_else(|| Params.get("url").and_then(|V| V.as_str()))
					.unwrap_or("")
					.to_string();
				// Emit to Sky — Sky uses Tauri shell plugin to open the URL
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://native/openExternal", json!({ "url": Url }));
				Ok(OkResponse(RequestId, &json!({ "success": true })))
			},
			// ---- Window (Cocoon MountainGRPCClient format) ----
			"showTextDocument" => {
				use tauri::Emitter;
				let Uri = Params
					.get("uri")
					.and_then(|V| V.get("value").or(Some(V)))
					.and_then(|V| V.as_str())
					.unwrap_or("")
					.to_string();
				let ViewColumn = Params.get("viewColumn").and_then(|V| V.as_i64()).map(|N| N + 2);
				let PreserveFocus = Params.get("preserveFocus").and_then(|V| V.as_bool()).unwrap_or(false);
				let _ = self.environment.ApplicationHandle.emit(
					"sky://editor/openDocument",
					json!({ "uri": Uri, "viewColumn": ViewColumn, "preserveFocus": PreserveFocus }),
				);
				Ok(OkResponse(RequestId, &json!({ "success": true })))
			},
			"showInformation" => {
					use CommonLibrary::UserInterface::{
						DTO::MessageSeverity::MessageSeverity,
						UserInterfaceProvider::UserInterfaceProvider,
					};
					let Message = Params.get("message").and_then(|V| V.as_str()).unwrap_or("").to_string();
					let Items:Option<serde_json::Value> = Params.get("items").cloned().filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());
					match self.environment.ShowMessage(MessageSeverity::Info, Message, Items).await {
						Ok(Some(Selected)) => Ok(OkResponse(RequestId, &json!({ "selectedItem": Selected }))),
						Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
						Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
					}
			},
			"showWarning" => {
					use CommonLibrary::UserInterface::{
						DTO::MessageSeverity::MessageSeverity,
						UserInterfaceProvider::UserInterfaceProvider,
					};
					let Message = Params.get("message").and_then(|V| V.as_str()).unwrap_or("").to_string();
					let Items:Option<serde_json::Value> = Params.get("items").cloned().filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());
					match self.environment.ShowMessage(MessageSeverity::Warning, Message, Items).await {
						Ok(Some(Selected)) => Ok(OkResponse(RequestId, &json!({ "selectedItem": Selected }))),
						Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
						Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
					}
			},
			"showError" => {
					use CommonLibrary::UserInterface::{
						DTO::MessageSeverity::MessageSeverity,
						UserInterfaceProvider::UserInterfaceProvider,
					};
					let Message = Params.get("message").and_then(|V| V.as_str()).unwrap_or("").to_string();
					let Items:Option<serde_json::Value> = Params.get("items").cloned().filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());
					match self.environment.ShowMessage(MessageSeverity::Error, Message, Items).await {
						Ok(Some(Selected)) => Ok(OkResponse(RequestId, &json!({ "selectedItem": Selected }))),
						Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
						Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
					}
			},
			"createStatusBarItem" => {
				use tauri::Emitter;
				let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Tooltip = Params.get("tooltip").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://statusbar/create", json!({ "id": Id, "text": Text, "tooltip": Tooltip }));
				Ok(OkResponse(RequestId, &json!({ "itemId": Id })))
			},
			"setStatusBarText" => {
				use tauri::Emitter;
				let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://statusbar/update", json!({ "id": ItemId, "text": Text }));
				Ok(OkResponse(RequestId, &json!({ "success": true })))
			},
			"createWebviewPanel" => {
				use tauri::Emitter;
				let ViewType = Params.get("viewType").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Title = Params.get("title").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Handle = std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)
					.map(|D| D.as_millis() as u64)
					.unwrap_or(0);
				let _ = self.environment.ApplicationHandle.emit("sky://webview/create", json!({ "handle": Handle, "viewType": ViewType, "title": Title, "viewColumn": Params.get("viewColumn"), "preserveFocus": Params.get("preserveFocus").and_then(|V| V.as_bool()).unwrap_or(false) }));
				Ok(OkResponse(RequestId, &json!({ "handle": Handle })))
			},
			"setWebviewHtml" => {
				use tauri::Emitter;
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0);
				let Html = Params.get("html").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://webview/setHtml", json!({ "handle": Handle, "html": Html }));
				Ok(OkResponse(RequestId, &json!({ "success": true })))
			},
			// ---- Workspace (Cocoon MountainGRPCClient format) ----
			"findFiles" => {
				use std::path::PathBuf;

				use globset::GlobBuilder;
				let Pattern = Params.get("pattern").and_then(|V| V.as_str()).unwrap_or("**").to_string();
				let WorkspaceFolders = self.environment.ApplicationState.Workspace.GetWorkspaceFolders();
				if WorkspaceFolders.is_empty() {
					return Ok(OkResponse(RequestId, &json!({ "uris": Vec::<String>::new() })));
				}
				let RootPath = PathBuf::from(&WorkspaceFolders[0].URI.to_string().replace("file://", ""));
				let Matcher = match GlobBuilder::new(&Pattern).literal_separator(false).build() {
					Ok(G) => G.compile_matcher(),
					Err(E) => return Ok(ErrResponse(RequestId, -32000, format!("Invalid glob: {}", E))),
				};
				let mut Files:Vec<String> = Vec::new();
				let mut Stack = vec![RootPath.clone()];
				'find_outer: while let Some(Dir) = Stack.pop() {
					let mut Entries = match tokio::fs::read_dir(&Dir).await {
						Ok(E) => E,
						Err(_) => continue,
					};
					while let Ok(Some(Entry)) = Entries.next_entry().await {
						let Path = Entry.path();
						if Path.file_name().map(|N| N.to_string_lossy().starts_with('.')).unwrap_or(false) {
							continue;
						}
						if Path.is_dir() {
							Stack.push(Path);
							continue;
						}
						let Rel = Path.strip_prefix(&RootPath).unwrap_or(&Path).to_string_lossy().to_string();
						if Matcher.is_match(&Rel) {
							Files.push(format!("file://{}", Path.to_string_lossy()));
							if Files.len() >= 500 {
								break 'find_outer;
							}
						}
					}
				}
				Ok(OkResponse(RequestId, &json!({ "uris": Files })))
			},
			"findTextInFiles" => {
				use std::path::PathBuf;

				use globset::GlobBuilder;
				let Pattern = Params.get("pattern").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let IncludeStr = Params
					.get("include")
					.and_then(|V| V.as_array())
					.and_then(|A| A.first())
					.and_then(|V| V.as_str())
					.map(|S| S.to_string())
					.unwrap_or_else(|| "**".to_string());
				let WorkspaceFolders = self.environment.ApplicationState.Workspace.GetWorkspaceFolders();
				if WorkspaceFolders.is_empty() {
					return Ok(OkResponse(RequestId, &json!({ "matches": Vec::<serde_json::Value>::new() })));
				}
				let RootPath = PathBuf::from(&WorkspaceFolders[0].URI.to_string().replace("file://", ""));
				let Matcher = GlobBuilder::new(&IncludeStr)
					.literal_separator(false)
					.build()
					.map(|G| G.compile_matcher())
					.ok();
				let PatternLower = Pattern.to_lowercase();
				let mut Matches:Vec<serde_json::Value> = Vec::new();
				let mut Stack = vec![RootPath.clone()];
				'text_outer: while let Some(Dir) = Stack.pop() {
					let mut Entries = match tokio::fs::read_dir(&Dir).await {
						Ok(E) => E,
						Err(_) => continue,
					};
					while let Ok(Some(Entry)) = Entries.next_entry().await {
						let Path = Entry.path();
						if Path.file_name().map(|N| N.to_string_lossy().starts_with('.')).unwrap_or(false) {
							continue;
						}
						if Path.is_dir() {
							Stack.push(Path);
							continue;
						}
						let Rel = Path.strip_prefix(&RootPath).unwrap_or(&Path).to_string_lossy().to_string();
						if let Some(Ref) = &Matcher {
							if !Ref.is_match(&Rel) {
								continue;
							}
						}
						let Content = match tokio::fs::read_to_string(&Path).await {
							Ok(C) => C,
							Err(_) => continue,
						};
						for (LineIdx, Line) in Content.lines().enumerate() {
							if Line.to_lowercase().contains(&PatternLower) {
								Matches.push(json!({ "uri": format!("file://{}", Path.to_string_lossy()), "lineNumber": LineIdx + 1, "preview": Line.trim() }));
								if Matches.len() >= 1000 {
									break 'text_outer;
								}
							}
						}
					}
				}
				Ok(OkResponse(RequestId, &json!({ "matches": Matches })))
			},
			"openDocument" => {
				use tauri::Emitter;
				let Uri = Params
					.get("uri")
					.and_then(|V| V.get("value").or(Some(V)))
					.and_then(|V| V.as_str())
					.unwrap_or("")
					.to_string();
				let ViewColumn = Params.get("viewColumn").and_then(|V| V.as_i64());
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://editor/openDocument", json!({ "uri": Uri, "viewColumn": ViewColumn }));
				Ok(OkResponse(RequestId, &json!({ "success": true })))
			},
			"saveAll" => {
				use tauri::Emitter;
				let IncludeUntitled = Params.get("includeUntitled").and_then(|V| V.as_bool()).unwrap_or(false);
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://editor/saveAll", json!({ "includeUntitled": IncludeUntitled }));
				Ok(OkResponse(RequestId, &json!({ "success": true })))
			},
			"applyEdit" => {
				use tauri::Emitter;
				let Uri = Params
					.get("uri")
					.and_then(|V| V.get("value").or(Some(V)))
					.and_then(|V| V.as_str())
					.unwrap_or("")
					.to_string();
				let Edits = Params.get("edits").cloned().unwrap_or(json!([]));
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://editor/applyEdits", json!({ "uri": Uri, "edits": Edits }));
				Ok(OkResponse(RequestId, &json!({ "success": true })))
			},
			// ---- Secret Storage (Cocoon MountainGRPCClient format) ----
			"getSecret" => {
				use CommonLibrary::Secret::SecretProvider::SecretProvider;
				let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();
				match self.environment.GetSecret(ExtensionId, Key).await {
					Ok(Some(Value)) => Ok(OkResponse(RequestId, &json!({ "value": Value }))),
					Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			"storeSecret" => {
				use CommonLibrary::Secret::SecretProvider::SecretProvider;
				let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Value = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();
				match self.environment.StoreSecret(ExtensionId, Key, Value).await {
					Ok(()) => Ok(OkResponse(RequestId, &json!({ "success": true }))),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			"deleteSecret" => {
				use CommonLibrary::Secret::SecretProvider::SecretProvider;
				let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();
				match self.environment.DeleteSecret(ExtensionId, Key).await {
					Ok(()) => Ok(OkResponse(RequestId, &json!({ "success": true }))),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
				}
			},
			// ---- FS aliases (Cocoon MountainGRPCClient uses different key names) ----
			"readFile" => {
				let Uri = Params
					.get("uri")
					.and_then(|V| V.as_str())
					.or_else(|| Params.as_str())
					.unwrap_or("")
					.replace("file://", "");
				match tokio::fs::read(&Uri).await {
					Ok(Content) => Ok(OkResponse(RequestId, &Content)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("readFile: {}", Error))),
				}
			},
			"writeFile" => {
				let Uri = Params.get("uri").and_then(|V| V.as_str()).unwrap_or("").replace("file://", "");
				let Content:Vec<u8> = Params
					.get("content")
					.and_then(|V| V.as_array())
					.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
					.unwrap_or_default();
				match tokio::fs::write(&Uri, &Content).await {
					Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("writeFile: {}", Error))),
				}
			},
			"stat" => {
				let Uri = Params
					.get("uri")
					.and_then(|V| V.as_str())
					.or_else(|| Params.as_str())
					.unwrap_or("")
					.replace("file://", "");
				match tokio::fs::metadata(&Uri).await {
					Ok(Meta) => {
						let Mtime = Meta
							.modified()
							.ok()
							.and_then(|T| T.duration_since(UNIX_EPOCH).ok())
							.map(|D| D.as_millis() as u64)
							.unwrap_or(0);
						Ok(OkResponse(
							RequestId,
							&json!({ "type": if Meta.is_dir() { 2 } else { 1 }, "is_file": Meta.is_file(), "is_directory": Meta.is_dir(), "size": Meta.len(), "mtime": Mtime }),
						))
					},
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("stat: {}", Error))),
				}
			},
			"readdir" => {
				let Uri = Params
					.get("uri")
					.and_then(|V| V.as_str())
					.or_else(|| Params.as_str())
					.unwrap_or("")
					.replace("file://", "");
				match tokio::fs::read_dir(&Uri).await {
					Ok(mut Entries) => {
						let mut Names:Vec<String> = Vec::new();
						while let Ok(Some(Entry)) = Entries.next_entry().await {
							if let Some(Name) = Entry.file_name().to_str() {
								Names.push(Name.to_string());
							}
						}
						Ok(OkResponse(RequestId, &Names))
					},
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("readdir: {}", Error))),
				}
			},
			// ---- Unknown ----
			_ => {
				warn!("[CocoonService] Unknown generic method: {}", Req.method);
				Ok(ErrResponse(RequestId, -32601, format!("Method '{}' not found", Req.method)))
			},
		}
	}

	/// Send Mountain notifications to Cocoon (generic fire-and-forget)
	/// Routes by notification.method string to the appropriate Mountain
	/// handler. Called by Cocoon's
	/// `MountainGRPCClient.sendNotification(method, params)`.
	async fn send_mountain_notification(
		&self,
		request:Request<GenericNotification>,
	) -> Result<Response<Empty>, Status> {
		let notification = request.into_inner();
		debug!("[CocoonService] Notification router: method='{}'", notification.method);

		// Deserialise notification parameters as JSON
		let Params:serde_json::Value = if notification.parameter.is_empty() {
			serde_json::Value::Null
		} else {
			serde_json::from_slice(&notification.parameter).unwrap_or(serde_json::Value::Null)
		};

		match notification.method.as_str() {
			// ---- Commands ----
			"registerCommand" => {
				let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				if let Err(Error) = self.environment.RegisterCommand(ExtensionId, CommandId.clone()).await {
					warn!(
						"[CocoonService] notification: registerCommand '{}' failed: {:?}",
						CommandId, Error
					);
				}
			},
			"unregisterCommand" => {
				let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self.environment.UnregisterCommand(ExtensionId, CommandId).await;
			},
			// ---- Language Providers (APIFactoryService.ts register_*_provider strings) ----
			"register_hover_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::Hover, Selector, ExtId);
			},
			"register_completion_item_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::Completion, Selector, ExtId);
			},
			"register_definition_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::Definition, Selector, ExtId);
			},
			"register_reference_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::References, Selector, ExtId);
			},
			"register_code_actions_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::CodeAction, Selector, ExtId);
			},
			"register_document_highlight_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::DocumentHighlight, Selector, ExtId);
			},
			"register_document_symbol_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::DocumentSymbol, Selector, ExtId);
			},
			"register_workspace_symbol_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::WorkspaceSymbol, Selector, ExtId);
			},
			"register_rename_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::Rename, Selector, ExtId);
			},
			"register_document_formatting_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::DocumentFormatting, Selector, ExtId);
			},
			"register_document_range_formatting_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::DocumentRangeFormatting, Selector, ExtId);
			},
			"register_on_type_formatting_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::OnTypeFormatting, Selector, ExtId);
			},
			"register_signature_help_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::SignatureHelp, Selector, ExtId);
			},
			"register_code_lens_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::CodeLens, Selector, ExtId);
			},
			"register_folding_range_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::FoldingRange, Selector, ExtId);
			},
			"register_selection_range_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::SelectionRange, Selector, ExtId);
			},
			"register_semantic_tokens_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::SemanticTokens, Selector, ExtId);
			},
			"register_inlay_hints_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::InlayHint, Selector, ExtId);
			},
			"register_type_hierarchy_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::TypeHierarchy, Selector, ExtId);
			},
			"register_call_hierarchy_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::CallHierarchy, Selector, ExtId);
			},
			"register_linked_editing_range_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::LinkedEditingRange, Selector, ExtId);
			},
			"register_document_link_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::DocumentLink, Selector, ExtId);
			},
			"register_color_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::Color, Selector, ExtId);
			},
			"register_implementation_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::Implementation, Selector, ExtId);
			},
			"register_type_definition_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::TypeDefinition, Selector, ExtId);
			},
			"register_declaration_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::Declaration, Selector, ExtId);
			},
			"register_evaluatable_expression_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::EvaluatableExpression, Selector, ExtId);
			},
			"register_inline_values_provider" => {
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
				let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");
				let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");
				self.RegisterProvider(Handle, ProviderType::InlineValues, Selector, ExtId);
			},
			// ---- Webview ----
			"onDidReceiveMessage" => {
				use tauri::Emitter;
				let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0);
				let Message = Params
					.get("stringMessage")
					.and_then(|V| V.as_str())
					.map(|S| S.to_string())
					.or_else(|| Params.get("bytesMessage").map(|_| "[binary]".to_string()))
					.unwrap_or_default();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://webview/message", json!({ "handle": Handle, "message": Message }));
			},
			// ---- Secrets (fire-and-forget variants) ----
			"storeSecret" => {
				use CommonLibrary::Secret::SecretProvider::SecretProvider;
				let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Value = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self.environment.StoreSecret(ExtensionId, Key, Value).await;
			},
			"deleteSecret" => {
				use CommonLibrary::Secret::SecretProvider::SecretProvider;
				let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self.environment.DeleteSecret(ExtensionId, Key).await;
			},
			// ---- File system (fire-and-forget write) ----
			"writeFile" => {
				let Uri = Params
					.get("uri")
					.and_then(|V| V.get("value").or(Some(V)))
					.and_then(|V| V.as_str())
					.unwrap_or("")
					.replace("file://", "");
				let Content:Vec<u8> = Params
					.get("content")
					.and_then(|V| V.as_array())
					.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
					.unwrap_or_default();
				let _ = tokio::fs::write(&Uri, &Content).await;
			},
			// ---- Webview panel ----
			"webview.postMessage" => {
				use tauri::Emitter;
				let PanelId = Params.get("panelId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Method = Params.get("method").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let MsgParams = Params.get("params").cloned().unwrap_or(serde_json::Value::Null);
				let _ = self.environment.ApplicationHandle.emit("sky://webview/message", json!({ "panelId": PanelId, "method": Method, "params": MsgParams }));
			},
			"webview.dispose" => {
				use tauri::Emitter;
				let PanelId = Params.get("panelId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self.environment.ApplicationHandle.emit("sky://webview/dispose", json!({ "panelId": PanelId }));
			},
			// ---- Progress indicator ----
			"progress.start" => {
				use tauri::Emitter;
				let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Title = Params.get("title").and_then(|V| V.as_str()).map(|S| S.to_string());
				let Location = Params.get("location").cloned();
				let Cancellable = Params.get("cancellable").and_then(|V| V.as_bool()).unwrap_or(false);
				let _ = self.environment.ApplicationHandle.emit("sky://progress/start", json!({ "id": Id, "title": Title, "location": Location, "cancellable": Cancellable }));
			},
			"progress.update" => {
				use tauri::Emitter;
				let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Message = Params.get("message").and_then(|V| V.as_str()).map(|S| S.to_string());
				let Increment = Params.get("increment").and_then(|V| V.as_f64());
				let _ = self.environment.ApplicationHandle.emit("sky://progress/update", json!({ "id": Id, "message": Message, "increment": Increment }));
			},
			"progress.complete" => {
				use tauri::Emitter;
				let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self.environment.ApplicationHandle.emit("sky://progress/complete", json!({ "id": Id }));
			},
			// ---- Native shell ----
			"openExternal" => {
				use tauri::Emitter;
				let Url = Params.get("url").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://native/openExternal", json!({ "url": Url }));
			},
			// ---- StatusBar updates (fire-and-forget from Window.ts setters) ----
			"setStatusBarText" | "statusBar.setText" => {
				use tauri::Emitter;
				let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://statusbar/update", json!({ "id": ItemId, "text": Text }));
			},
			"disposeStatusBarItem" | "statusBar.dispose" => {
				use tauri::Emitter;
				let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://statusbar/dispose", json!({ "id": ItemId }));
			},
			// ---- Output channel (fire-and-forget from Window.ts OutputChannel proxy) ----
			"output.create" => {
				use tauri::Emitter;
				let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Name = Params.get("name").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://output/create", json!({ "id": Id, "name": Name }));
			},
			"output.append" => {
				use tauri::Emitter;
				let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Text = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://output/append", json!({ "channel": Channel, "text": Text }));
			},
			"output.appendLine" => {
				use tauri::Emitter;
				let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Line = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self.environment.ApplicationHandle.emit(
					"sky://output/append",
					json!({ "channel": Channel, "text": format!("{}\n", Line) }),
				);
			},
			"output.clear" => {
				use tauri::Emitter;
				let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://output/clear", json!({ "channel": Channel }));
			},
			"output.show" => {
				use tauri::Emitter;
				let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://output/show", json!({ "channel": Channel }));
			},
			"output.dispose" => {
				use tauri::Emitter;
				let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://output/dispose", json!({ "channel": Channel }));
			},
			// ---- Language configuration ----
			"set_language_configuration" => {
				// Language configuration is consumed by Sky — emit for workbench to pick up
				use tauri::Emitter;
				let Language = Params.get("language").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://language/configure", json!({ "language": Language }));
			},
			_ => {
				debug!("[CocoonService] Unknown notification method: '{}'", notification.method);
			},
		}

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

		// Store workspace folders from the init payload into ApplicationState
		let Folders:Vec<WorkspaceFolderStateDTO> = req
			.workspace_folders
			.iter()
			.enumerate()
			.filter_map(|(Index, F)| {
				let UriValue = F.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");
				url::Url::parse(UriValue)
					.ok()
					.and_then(|ParsedUrl| WorkspaceFolderStateDTO::New(ParsedUrl, F.name.clone(), Index).ok())
			})
			.collect();

		if !Folders.is_empty() {
			self.environment.ApplicationState.Workspace.SetWorkspaceFolders(Folders);
			debug!("[CocoonService] Workspace folders stored: {}", req.workspace_folders.len());
		}

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

		// Wire to CommandExecutor::RegisterCommand which stores a Proxied handler
		// pointing back to the Cocoon sidecar.
		if let Err(Error) = self
			.environment
			.RegisterCommand(req.extension_id.clone(), req.command_id.clone())
			.await
		{
			warn!("[CocoonService] Failed to register command '{}': {:?}", req.command_id, Error);
		} else {
			debug!(
				"[CocoonService] Command registered: id={}, title={:?}",
				req.command_id, req.title
			);
		}

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

		// Convert the first Argument oneof value to a serde_json::Value
		let Arg:serde_json::Value = req
			.arguments
			.first()
			.and_then(|A| A.value.as_ref())
			.map(|V| {
				match V {
					crate::Vine::Generated::argument::Value::StringValue(S) => json!(S),
					crate::Vine::Generated::argument::Value::IntValue(I) => json!(I),
					crate::Vine::Generated::argument::Value::BoolValue(B) => json!(B),
					crate::Vine::Generated::argument::Value::BytesValue(Bytes) => {
						serde_json::from_slice(Bytes).unwrap_or(serde_json::Value::Null)
					},
				}
			})
			.unwrap_or(serde_json::Value::Null);

		match self.environment.ExecuteCommand(req.command_id, Arg).await {
			Ok(Value) => {
				let Bytes = serde_json::to_vec(&Value).unwrap_or_default();
				Ok(Response::new(ExecuteCommandResponse {
					result:Some(crate::Vine::Generated::execute_command_response::Result::Value(Bytes)),
				}))
			},
			Err(Error) => {
				let Bytes = serde_json::to_vec(&Error.to_string()).unwrap_or_default();
				Ok(Response::new(ExecuteCommandResponse {
					result:Some(crate::Vine::Generated::execute_command_response::Result::Error(
						crate::Vine::Generated::RpcError { code:-32000, message:Error.to_string(), data:Bytes },
					)),
				}))
			},
		}
	}

	/// Unregister Command - Unregister a previously registered command
	async fn unregister_command(&self, request:Request<UnregisterCommandRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Unregistering command '{}'", req.command_id);

		// Wire to CommandExecutor::UnregisterCommand
		if let Err(Error) = self.environment.UnregisterCommand(String::new(), req.command_id.clone()).await {
			warn!("[CocoonService] Failed to unregister command '{}': {:?}", req.command_id, Error);
		} else {
			debug!("[CocoonService] Command removed: {}", req.command_id);
		}

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
		self.RegisterProvider(req.handle, ProviderType::Hover, &req.language_selector, &req.extension_id);
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
		self.RegisterProvider(req.handle, ProviderType::Completion, &req.language_selector, &req.extension_id);
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
		self.RegisterProvider(req.handle, ProviderType::Definition, &req.language_selector, &req.extension_id);
		// TODO: When ProviderRegistry is available in MountainEnvironment:
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
		self.RegisterProvider(req.handle, ProviderType::References, &req.language_selector, &req.extension_id);
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
		self.RegisterProvider(req.handle, ProviderType::CodeAction, &req.language_selector, &req.extension_id);
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

	/// Show Text Document — emit a Tauri event so Sky opens the document tab.
	async fn show_text_document(
		&self,
		request:Request<ShowTextDocumentRequest>,
	) -> Result<Response<ShowTextDocumentResponse>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		let Uri = req.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();
		info!("[CocoonService] show_text_document: {}", Uri);

		let _ = self.environment.ApplicationHandle.emit(
			"sky://editor/openDocument",
			json!({ "uri": Uri, "viewColumn": req.view_column }),
		);

		Ok(Response::new(ShowTextDocumentResponse { success:true }))
	}

	/// Show Information Message — delegate to
	/// UserInterfaceProvider::ShowMessage.
	async fn show_information_message(
		&self,
		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		use CommonLibrary::UserInterface::{
			DTO::MessageSeverity::MessageSeverity,
			UserInterfaceProvider::UserInterfaceProvider,
		};

		let req = request.into_inner();
		info!("[CocoonService] show_information_message: {}", req.message);

		let _ = self.environment.ShowMessage(MessageSeverity::Info, req.message, None).await;

		Ok(Response::new(ShowMessageResponse { success:true }))
	}

	/// Show Warning Message — delegate to UserInterfaceProvider::ShowMessage.
	async fn show_warning_message(
		&self,
		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		use CommonLibrary::UserInterface::{
			DTO::MessageSeverity::MessageSeverity,
			UserInterfaceProvider::UserInterfaceProvider,
		};

		let req = request.into_inner();
		warn!("[CocoonService] show_warning_message: {}", req.message);

		let _ = self.environment.ShowMessage(MessageSeverity::Warning, req.message, None).await;

		Ok(Response::new(ShowMessageResponse { success:true }))
	}

	/// Show Error Message — delegate to UserInterfaceProvider::ShowMessage.
	async fn show_error_message(
		&self,
		request:Request<ShowMessageRequest>,
	) -> Result<Response<ShowMessageResponse>, Status> {
		use CommonLibrary::UserInterface::{
			DTO::MessageSeverity::MessageSeverity,
			UserInterfaceProvider::UserInterfaceProvider,
		};

		let req = request.into_inner();
		error!("[CocoonService] show_error_message: {}", req.message);

		let _ = self.environment.ShowMessage(MessageSeverity::Error, req.message, None).await;

		Ok(Response::new(ShowMessageResponse { success:true }))
	}

	/// Create Status Bar Item — emit Tauri event for Sky to render status bar
	/// entry.
	async fn create_status_bar_item(
		&self,
		request:Request<CreateStatusBarItemRequest>,
	) -> Result<Response<CreateStatusBarItemResponse>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		info!("[CocoonService] create_status_bar_item: {}", req.id);

		let _ = self.environment.ApplicationHandle.emit(
			"sky://statusbar/create",
			json!({ "id": req.id, "text": req.text, "tooltip": req.tooltip }),
		);

		Ok(Response::new(CreateStatusBarItemResponse { item_id:req.id.clone() }))
	}

	/// Set Status Bar Text — emit Tauri event for Sky status bar update.
	async fn set_status_bar_text(&self, request:Request<SetStatusBarTextRequest>) -> Result<Response<Empty>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		debug!("[CocoonService] set_status_bar_text: id={} text={}", req.item_id, req.text);

		let _ = self
			.environment
			.ApplicationHandle
			.emit("sky://statusbar/update", json!({ "id": req.item_id, "text": req.text }));

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
		let Path = Self::UriToPath(req.uri.as_ref())
			.ok_or_else(|| Status::invalid_argument("read_file: missing or empty URI"))?;

		debug!("[CocoonService] Reading file: {:?}", Path);

		let Content = tokio::fs::read(&Path).await.map_err(|Error| {
			warn!("[CocoonService] read_file failed for {:?}: {}", Path, Error);
			Status::not_found(format!("read_file: {}: {}", Path.display(), Error))
		})?;

		Ok(Response::new(ReadFileResponse {
			content:Content,
			encoding:"utf-8".to_string(),
		}))
	}

	/// Write File - Write file contents
	async fn write_file(&self, request:Request<WriteFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let Path = Self::UriToPath(req.uri.as_ref())
			.ok_or_else(|| Status::invalid_argument("write_file: missing or empty URI"))?;

		debug!("[CocoonService] Writing file: {:?} ({} bytes)", Path, req.content.len());

		// Ensure parent directory exists
		if let Some(Parent) = Path.parent() {
			if !Parent.as_os_str().is_empty() {
				tokio::fs::create_dir_all(Parent)
					.await
					.map_err(|Error| Status::internal(format!("write_file: create_dir_all {:?}: {}", Parent, Error)))?;
			}
		}

		tokio::fs::write(&Path, &req.content).await.map_err(|Error| {
			warn!("[CocoonService] write_file failed for {:?}: {}", Path, Error);
			Status::internal(format!("write_file: {}: {}", Path.display(), Error))
		})?;

		Ok(Response::new(Empty {}))
	}

	/// Stat - Get file metadata
	async fn stat(&self, request:Request<StatRequest>) -> Result<Response<StatResponse>, Status> {
		let req = request.into_inner();
		let Path =
			Self::UriToPath(req.uri.as_ref()).ok_or_else(|| Status::invalid_argument("stat: missing or empty URI"))?;

		debug!("[CocoonService] Stat: {:?}", Path);

		let Metadata = tokio::fs::metadata(&Path).await.map_err(|Error| {
			warn!("[CocoonService] stat failed for {:?}: {}", Path, Error);
			Status::not_found(format!("stat: {}: {}", Path.display(), Error))
		})?;

		let Mtime = Metadata
			.modified()
			.ok()
			.and_then(|T| T.duration_since(UNIX_EPOCH).ok())
			.map(|D| D.as_millis() as u64)
			.unwrap_or(0);

		Ok(Response::new(StatResponse {
			is_file:Metadata.is_file(),
			is_directory:Metadata.is_dir(),
			size:Metadata.len(),
			mtime:Mtime,
		}))
	}

	/// Read Directory - List directory contents
	async fn readdir(&self, request:Request<ReaddirRequest>) -> Result<Response<ReaddirResponse>, Status> {
		let req = request.into_inner();
		let Path = Self::UriToPath(req.uri.as_ref())
			.ok_or_else(|| Status::invalid_argument("readdir: missing or empty URI"))?;

		debug!("[CocoonService] Readdir: {:?}", Path);

		let mut ReadDir = tokio::fs::read_dir(&Path).await.map_err(|Error| {
			warn!("[CocoonService] readdir failed for {:?}: {}", Path, Error);
			Status::not_found(format!("readdir: {}: {}", Path.display(), Error))
		})?;

		let mut Entries = Vec::new();
		while let Ok(Some(Entry)) = ReadDir.next_entry().await {
			if let Some(Name) = Entry.file_name().to_str() {
				Entries.push(Name.to_string());
			}
		}

		Ok(Response::new(ReaddirResponse { entries:Entries }))
	}

	/// Watch File - Watch file for changes
	///
	/// Logs the watch request. Full inotify/FSEvents integration is a P1 task
	/// requiring the `notify` crate wired into ApplicationState.
	async fn watch_file(&self, request:Request<WatchFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let Uri = req.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");
		info!("[CocoonService] watch_file registered (polling not yet active): {}", Uri);
		// TODO(P1): Wire notify crate watcher; store WatcherHandle in
		// ApplicationState.Feature.Watchers keyed by URI for cancellation on
		// cancel_operation.
		Ok(Response::new(Empty {}))
	}

	// ==================== Workspace Operations ====================

	/// Find Files - Search for files using glob pattern across workspace
	/// folders.
	async fn find_files(&self, request:Request<FindFilesRequest>) -> Result<Response<FindFilesResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] Finding files with pattern: {}", req.pattern);

		use globset::{Glob, GlobSetBuilder};

		// Build glob matcher
		let GlobMatcher = Glob::new(&req.pattern)
			.map_err(|Error| {
				Status::invalid_argument(format!("find_files: invalid pattern '{}': {}", req.pattern, Error))
			})?
			.compile_matcher();

		// Collect workspace root folders from ApplicationState
		let Roots:Vec<std::path::PathBuf> = {
			match self.environment.ApplicationState.Workspace.WorkspaceFolders.lock() {
				Ok(Guard) => Guard.iter().map(|F| std::path::PathBuf::from(F.URI.path())).collect(),
				Err(_) => Vec::new(),
			}
		};

		let SearchRoots = if Roots.is_empty() {
			vec![std::env::current_dir().unwrap_or_default()]
		} else {
			Roots
		};

		// Walk each root and collect matching paths
		let mut Uris = Vec::new();

		fn WalkAndCollect(
			Directory:&std::path::Path,
			Root:&std::path::Path,
			Matcher:&globset::GlobMatcher,
			Results:&mut Vec<String>,
		) {
			if let Ok(Entries) = std::fs::read_dir(Directory) {
				for Entry in Entries.flatten() {
					let EntryPath = Entry.path();
					if EntryPath.is_dir() {
						WalkAndCollect(&EntryPath, Root, Matcher, Results);
					} else if let Ok(Relative) = EntryPath.strip_prefix(Root) {
						if Matcher.is_match(Relative) {
							Results.push(format!("file://{}", EntryPath.display()));
						}
					}
				}
			}
		}

		for Root in &SearchRoots {
			WalkAndCollect(Root, Root, &GlobMatcher, &mut Uris);
		}

		debug!(
			"[CocoonService] find_files: {} results for pattern '{}'",
			Uris.len(),
			req.pattern
		);
		Ok(Response::new(FindFilesResponse { uris:Uris }))
	}

	/// Find Text in Files — walk workspace and grep for pattern.
	///
	/// Uses a simple line-by-line scan (not indexed). Returns up to 1000
	/// matches. Indexing integration is a P2 task.
	async fn find_text_in_files(
		&self,
		request:Request<FindTextInFilesRequest>,
	) -> Result<Response<FindTextInFilesResponse>, Status> {
		let req = request.into_inner();
		if req.pattern.is_empty() {
			return Ok(Response::new(FindTextInFilesResponse::default()));
		}
		debug!("[CocoonService] find_text_in_files: pattern='{}'", req.pattern);

		let Roots:Vec<std::path::PathBuf> = {
			match self.environment.ApplicationState.Workspace.WorkspaceFolders.lock() {
				Ok(Guard) => Guard.iter().map(|F| std::path::PathBuf::from(F.URI.path())).collect(),
				Err(_) => Vec::new(),
			}
		};
		let SearchRoots = if Roots.is_empty() {
			vec![std::env::current_dir().unwrap_or_default()]
		} else {
			Roots
		};

		let Pattern = req.pattern.clone();
		let Matches = tokio::task::spawn_blocking(move || {
			let mut Results:Vec<TextMatch> = Vec::new();
			const MAX_MATCHES:usize = 1000;

			fn WalkAndSearch(Dir:&std::path::Path, Pattern:&str, Results:&mut Vec<TextMatch>) {
				if Results.len() >= 1000 {
					return;
				}
				if let Ok(Entries) = std::fs::read_dir(Dir) {
					for Entry in Entries.flatten() {
						if Results.len() >= MAX_MATCHES {
							break;
						}
						let Path = Entry.path();
						if Path.is_dir() {
							// Skip hidden dirs and common noise dirs
							let DirName = Path.file_name().and_then(|N| N.to_str()).unwrap_or("");
							if DirName.starts_with('.') || DirName == "node_modules" || DirName == "target" {
								continue;
							}
							WalkAndSearch(&Path, Pattern, Results);
						} else if Path.is_file() {
							if let Ok(Content) = std::fs::read_to_string(&Path) {
								for (LineIdx, Line) in Content.lines().enumerate() {
									if Results.len() >= MAX_MATCHES {
										break;
									}
									if let Some(ColIdx) = Line.find(Pattern) {
										Results.push(TextMatch {
											uri:Some(Uri { value:format!("file://{}", Path.display()) }),
											range:Some(Range {
												start:Some(Position { line:LineIdx as u32, character:ColIdx as u32 }),
												end:Some(Position {
													line:LineIdx as u32,
													character:(ColIdx + Pattern.len()) as u32,
												}),
											}),
											preview:Line.to_string(),
										});
									}
								}
							}
						}
					}
				}
			}

			for Root in &SearchRoots {
				WalkAndSearch(Root, &Pattern, &mut Results);
				if Results.len() >= MAX_MATCHES {
					break;
				}
			}
			Results
		})
		.await
		.unwrap_or_default();

		debug!(
			"[CocoonService] find_text_in_files: {} matches for '{}'",
			Matches.len(),
			req.pattern
		);
		Ok(Response::new(FindTextInFilesResponse { matches:Matches }))
	}

	/// Open Document — emit Tauri event for Sky to open the editor tab.
	async fn open_document(
		&self,
		request:Request<OpenDocumentRequest>,
	) -> Result<Response<OpenDocumentResponse>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		let Uri = req.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();
		info!("[CocoonService] open_document: {}", Uri);

		let _ = self.environment.ApplicationHandle.emit(
			"sky://editor/openDocument",
			json!({ "uri": Uri, "viewColumn": req.view_column }),
		);

		Ok(Response::new(OpenDocumentResponse { success:true }))
	}

	/// Save All — emit Tauri event for Sky to save all open documents.
	async fn save_all(&self, request:Request<SaveAllRequest>) -> Result<Response<SaveAllResponse>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		info!("[CocoonService] save_all: includeUntitled={}", req.include_untitled);

		let _ = self
			.environment
			.ApplicationHandle
			.emit("sky://editor/saveAll", json!({ "includeUntitled": req.include_untitled }));

		Ok(Response::new(SaveAllResponse { success:true }))
	}

	/// Apply Edit — emit Tauri event for Sky to apply text edits in the editor.
	async fn apply_edit(&self, request:Request<ApplyEditRequest>) -> Result<Response<ApplyEditResponse>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		let Uri = req.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();
		debug!("[CocoonService] apply_edit: uri={} edits={}", Uri, req.edits.len());

		let EditsJson:Vec<serde_json::Value> = req.edits.iter().map(|E| {
			json!({
				"range": {
					"start": E.range.as_ref().and_then(|R| R.start.as_ref()).map(|P| json!({ "line": P.line, "character": P.character })),
					"end": E.range.as_ref().and_then(|R| R.end.as_ref()).map(|P| json!({ "line": P.line, "character": P.character })),
				},
				"newText": E.new_text,
			})
		}).collect();

		let _ = self
			.environment
			.ApplicationHandle
			.emit("sky://editor/applyEdits", json!({ "uri": Uri, "edits": EditsJson }));

		Ok(Response::new(ApplyEditResponse { success:true }))
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

		// Apply additions and removals to ApplicationState.Workspace
		{
			let mut Folders = self.environment.ApplicationState.Workspace.GetWorkspaceFolders();

			// Remove by URI
			let RemovalUris:Vec<String> = req
				.removals
				.iter()
				.filter_map(|F| F.uri.as_ref().map(|U| U.value.clone()))
				.collect();
			Folders.retain(|F| !RemovalUris.contains(&F.URI.to_string()));

			// Append additions
			let ExistingCount = Folders.len();
			for (Idx, Addition) in req.additions.iter().enumerate() {
				let UriValue = Addition.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");
				if let Ok(ParsedUrl) = url::Url::parse(UriValue) {
					if let Ok(DTO) = WorkspaceFolderStateDTO::New(ParsedUrl, Addition.name.clone(), ExistingCount + Idx)
					{
						Folders.push(DTO);
					}
				}
			}

			self.environment.ApplicationState.Workspace.SetWorkspaceFolders(Folders);
		}

		Ok(Response::new(Empty {}))
	}

	// ==================== Terminal ====================

	/// Open Terminal — create PTY via TerminalProvider and return the terminal
	/// ID.
	async fn open_terminal(&self, request:Request<OpenTerminalRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Opening terminal: {}", req.name);

		// Build options JSON matching TerminalStateDTO::Create expectations
		let Options = json!({
			"name": req.name,
			"shellPath": if req.shell_path.is_empty() { serde_json::Value::Null } else { json!(req.shell_path) },
			"shellArgs": req.shell_args,
			"cwd": if req.cwd.is_empty() { serde_json::Value::Null } else { json!(req.cwd) },
		});

		match self.environment.CreateTerminal(Options).await {
			Ok(Info) => {
				info!("[CocoonService] Terminal created: {:?}", Info);
				Ok(Response::new(Empty {}))
			},
			Err(Error) => {
				error!("[CocoonService] open_terminal failed: {}", Error);
				Err(Status::internal(format!("open_terminal: {}", Error)))
			},
		}
	}

	/// Terminal Input — write bytes to PTY stdin via TerminalProvider.
	async fn terminal_input(&self, request:Request<TerminalInputRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let TerminalId = req.terminal_id as u64;
		debug!("[CocoonService] terminal_input: id={} bytes={}", TerminalId, req.data.len());

		let Text = String::from_utf8_lossy(&req.data).into_owned();

		match self.environment.SendTextToTerminal(TerminalId, Text).await {
			Ok(()) => Ok(Response::new(Empty {})),
			Err(Error) => {
				warn!("[CocoonService] terminal_input failed id={}: {}", TerminalId, Error);
				Err(Status::not_found(format!("terminal_input: {}", Error)))
			},
		}
	}

	/// Close Terminal — dispose PTY and cleanup resources via TerminalProvider.
	async fn close_terminal(&self, request:Request<CloseTerminalRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let TerminalId = req.terminal_id as u64;
		info!("[CocoonService] close_terminal: id={}", TerminalId);

		match self.environment.DisposeTerminal(TerminalId).await {
			Ok(()) => Ok(Response::new(Empty {})),
			Err(Error) => {
				warn!("[CocoonService] close_terminal failed id={}: {}", TerminalId, Error);
				Err(Status::internal(format!("close_terminal: {}", Error)))
			},
		}
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

	/// Execute Git - Spawn native `git` process with provided arguments.
	///
	/// Runs git in the repository directory supplied by the extension host,
	/// captures stdout/stderr, and returns the raw bytes. Mirrors VS Code's
	/// `$gitExec` IPC call used by the built-in Git extension.
	async fn git_exec(&self, request:Request<GitExecRequest>) -> Result<Response<GitExecResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] git_exec: {}", req.args.join(" "));

		let WorkingDir = if req.repository_path.is_empty() {
			std::env::current_dir().unwrap_or_default()
		} else {
			std::path::PathBuf::from(&req.repository_path)
		};

		let Output = tokio::process::Command::new("git")
			.args(&req.args)
			.current_dir(&WorkingDir)
			.output()
			.await
			.map_err(|Error| {
				error!("[CocoonService] git_exec failed to spawn: {}", Error);
				Status::internal(format!("git_exec: failed to spawn git: {}", Error))
			})?;

		let ExitCode = Output.status.code().unwrap_or(-1);
		debug!(
			"[CocoonService] git_exec exit={} stdout={} bytes stderr={} bytes",
			ExitCode,
			Output.stdout.len(),
			Output.stderr.len()
		);

		// Combine stdout lines into repeated string output; prepend stderr lines
		// with "stderr: " prefix so extension can differentiate them.
		let StdoutStr = String::from_utf8_lossy(&Output.stdout);
		let StderrStr = String::from_utf8_lossy(&Output.stderr);
		let mut OutputLines:Vec<String> = StdoutStr.lines().map(|L| L.to_string()).collect();
		for Line in StderrStr.lines() {
			OutputLines.push(format!("stderr: {}", Line));
		}

		Ok(Response::new(GitExecResponse { output:OutputLines, exit_code:ExitCode }))
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

	/// Get Secret — retrieve from the OS keychain via SecretProvider.
	async fn get_secret(&self, request:Request<GetSecretRequest>) -> Result<Response<GetSecretResponse>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] get_secret: key={}", req.key);

		// The gRPC proto only carries `key`; we use the app name as the
		// extension identifier (keyring service scoping).
		match self.environment.GetSecret(String::new(), req.key.clone()).await {
			Ok(Some(Value)) => Ok(Response::new(GetSecretResponse { value:Value })),
			Ok(None) => Ok(Response::new(GetSecretResponse { value:String::new() })),
			Err(Error) => {
				warn!("[CocoonService] get_secret failed key={}: {}", req.key, Error);
				Err(Status::internal(format!("get_secret: {}", Error)))
			},
		}
	}

	/// Store Secret — persist to the OS keychain via SecretProvider.
	async fn store_secret(&self, request:Request<StoreSecretRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] store_secret: key={}", req.key);

		match self.environment.StoreSecret(String::new(), req.key.clone(), req.value).await {
			Ok(()) => Ok(Response::new(Empty {})),
			Err(Error) => {
				warn!("[CocoonService] store_secret failed key={}: {}", req.key, Error);
				Err(Status::internal(format!("store_secret: {}", Error)))
			},
		}
	}

	/// Delete Secret — remove from the OS keychain via SecretProvider.
	async fn delete_secret(&self, request:Request<DeleteSecretRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		debug!("[CocoonService] delete_secret: key={}", req.key);

		match self.environment.DeleteSecret(String::new(), req.key.clone()).await {
			Ok(()) => Ok(Response::new(Empty {})),
			Err(Error) => {
				warn!("[CocoonService] delete_secret failed key={}: {}", req.key, Error);
				Err(Status::internal(format!("delete_secret: {}", Error)))
			},
		}
	}

	// ==================== Extended Language Provider Handlers ====================

	/// Document Highlight Provider - Register
	async fn register_document_highlight_provider(
		&self,
		request:Request<RegisterProviderRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] Registering Document Highlight Provider");
		self.RegisterProvider(
			req.handle,
			ProviderType::DocumentHighlight,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::DocumentSymbol,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::WorkspaceSymbol,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(req.handle, ProviderType::Rename, &req.language_selector, &req.extension_id);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::DocumentFormatting,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::DocumentRangeFormatting,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::OnTypeFormatting,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::SignatureHelp,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(req.handle, ProviderType::CodeLens, &req.language_selector, &req.extension_id);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::FoldingRange,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::SelectionRange,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::SemanticTokens,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(req.handle, ProviderType::InlayHint, &req.language_selector, &req.extension_id);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::TypeHierarchy,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::CallHierarchy,
			&req.language_selector,
			&req.extension_id,
		);
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
		self.RegisterProvider(
			req.handle,
			ProviderType::LinkedEditingRange,
			&req.language_selector,
			&req.extension_id,
		);
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

	/// Show Quick Pick — present a selection list via UserInterfaceProvider.
	async fn show_quick_pick(
		&self,
		request:Request<ShowQuickPickRequest>,
	) -> Result<Response<ShowQuickPickResponse>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] show_quick_pick: {} items", req.items.len());

		let Items:Vec<QuickPickItemDTO> = req
			.items
			.iter()
			.map(|Item| {
				QuickPickItemDTO {
					Label:Item.label.clone(),
					Description:if Item.description.is_empty() { None } else { Some(Item.description.clone()) },
					Detail:None,
					Picked:Some(Item.picked),
					AlwaysShow:None,
				}
			})
			.collect();

		let Options = Some(QuickPickOptionsDTO {
			Title:if req.title.is_empty() { None } else { Some(req.title) },
			PlaceHolder:if req.placeholder.is_empty() { None } else { Some(req.placeholder) },
			CanPickMany:Some(req.can_pick_many),
			IgnoreFocusOut:None,
		});

		match self.environment.ShowQuickPick(Items, Options).await {
			Ok(Some(Selected)) => {
				// Map selected label strings back to indices via linear search
				let SelectedIndices:Vec<u32> = Selected
					.iter()
					.filter_map(|Label| req.items.iter().position(|Item| &Item.label == Label).map(|Idx| Idx as u32))
					.collect();
				Ok(Response::new(ShowQuickPickResponse { selected_indices:SelectedIndices }))
			},
			Ok(None) => Ok(Response::new(ShowQuickPickResponse::default())),
			Err(Error) => {
				warn!("[CocoonService] show_quick_pick failed: {}", Error);
				Ok(Response::new(ShowQuickPickResponse::default()))
			},
		}
	}

	/// Show Input Box — present a text entry dialog via UserInterfaceProvider.
	async fn show_input_box(
		&self,
		request:Request<ShowInputBoxRequest>,
	) -> Result<Response<ShowInputBoxResponse>, Status> {
		let req = request.into_inner();
		info!("[CocoonService] show_input_box");

		let Options = Some(InputBoxOptionsDTO {
			Title:if req.title.is_empty() { None } else { Some(req.title) },
			PlaceHolder:if req.placeholder.is_empty() { None } else { Some(req.placeholder) },
			Value:if req.value.is_empty() { None } else { Some(req.value) },
			Prompt:if req.prompt.is_empty() { None } else { Some(req.prompt) },
			IsPassword:if req.password { Some(true) } else { None },
			IgnoreFocusOut:None,
		});

		match self.environment.ShowInputBox(Options).await {
			Ok(Some(Value)) => Ok(Response::new(ShowInputBoxResponse { value:Value, cancelled:false })),
			Ok(None) => Ok(Response::new(ShowInputBoxResponse { value:String::new(), cancelled:true })),
			Err(Error) => {
				warn!("[CocoonService] show_input_box failed: {}", Error);
				Ok(Response::new(ShowInputBoxResponse { value:String::new(), cancelled:true }))
			},
		}
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
		let req = request.into_inner();
		let Path =
			Self::UriToPath(req.uri.as_ref()).ok_or_else(|| Status::invalid_argument("delete_file: missing URI"))?;

		debug!("[CocoonService] delete_file: {:?}", Path);

		if Path.is_dir() {
			tokio::fs::remove_dir_all(&Path).await
		} else {
			tokio::fs::remove_file(&Path).await
		}
		.map_err(|Error| Status::internal(format!("delete_file: {}: {}", Path.display(), Error)))?;

		Ok(Response::new(Empty {}))
	}

	/// file rename (move)
	async fn rename_file(&self, request:Request<RenameFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let OldPath = Self::UriToPath(req.source.as_ref())
			.ok_or_else(|| Status::invalid_argument("rename_file: missing source URI"))?;
		let NewPath = Self::UriToPath(req.target.as_ref())
			.ok_or_else(|| Status::invalid_argument("rename_file: missing target URI"))?;

		debug!("[CocoonService] rename_file: {:?} → {:?}", OldPath, NewPath);

		if let Some(Parent) = NewPath.parent() {
			if !Parent.as_os_str().is_empty() {
				tokio::fs::create_dir_all(Parent)
					.await
					.map_err(|Error| Status::internal(format!("rename_file: create_dir_all failed: {}", Error)))?;
			}
		}

		tokio::fs::rename(&OldPath, &NewPath)
			.await
			.map_err(|Error| Status::internal(format!("rename_file: {}: {}", OldPath.display(), Error)))?;

		Ok(Response::new(Empty {}))
	}

	/// file copy
	async fn copy_file(&self, request:Request<CopyFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let SrcPath = Self::UriToPath(req.source.as_ref())
			.ok_or_else(|| Status::invalid_argument("copy_file: missing source URI"))?;
		let DstPath = Self::UriToPath(req.target.as_ref())
			.ok_or_else(|| Status::invalid_argument("copy_file: missing target URI"))?;

		debug!("[CocoonService] copy_file: {:?} → {:?}", SrcPath, DstPath);

		if let Some(Parent) = DstPath.parent() {
			if !Parent.as_os_str().is_empty() {
				tokio::fs::create_dir_all(Parent)
					.await
					.map_err(|Error| Status::internal(format!("copy_file: create_dir_all failed: {}", Error)))?;
			}
		}

		tokio::fs::copy(&SrcPath, &DstPath)
			.await
			.map_err(|Error| Status::internal(format!("copy_file: {}: {}", SrcPath.display(), Error)))?;

		Ok(Response::new(Empty {}))
	}

	/// directory creation
	async fn create_directory(&self, request:Request<CreateDirectoryRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let Path = Self::UriToPath(req.uri.as_ref())
			.ok_or_else(|| Status::invalid_argument("create_directory: missing URI"))?;

		debug!("[CocoonService] create_directory: {:?}", Path);

		tokio::fs::create_dir_all(&Path)
			.await
			.map_err(|Error| Status::internal(format!("create_directory: {}: {}", Path.display(), Error)))?;

		Ok(Response::new(Empty {}))
	}

	/// Create output channel — notify Sky to create a named output panel.
	async fn create_output_channel(
		&self,
		request:Request<CreateOutputChannelRequest>,
	) -> Result<Response<CreateOutputChannelResponse>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		info!("[CocoonService] create_output_channel: '{}'", req.name);

		let _ = self
			.environment
			.ApplicationHandle
			.emit("sky://output/create", json!({ "channel": req.name }));

		Ok(Response::new(CreateOutputChannelResponse { channel_id:req.name.clone() }))
	}

	/// Append text to an output channel panel.
	async fn append_output(&self, request:Request<AppendOutputRequest>) -> Result<Response<Empty>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		let _ = self
			.environment
			.ApplicationHandle
			.emit("sky://output/append", json!({ "channel": req.channel_id, "text": req.value }));
		Ok(Response::new(Empty {}))
	}

	/// Clear an output channel panel.
	async fn clear_output(&self, request:Request<ClearOutputRequest>) -> Result<Response<Empty>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		let _ = self
			.environment
			.ApplicationHandle
			.emit("sky://output/clear", json!({ "channel": req.channel_id }));
		Ok(Response::new(Empty {}))
	}

	/// Show an output channel panel.
	async fn show_output(&self, request:Request<ShowOutputRequest>) -> Result<Response<Empty>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		let _ = self
			.environment
			.ApplicationHandle
			.emit("sky://output/show", json!({ "channel": req.channel_id }));
		Ok(Response::new(Empty {}))
	}

	/// Dispose an output channel (no cleanup needed; Sky removes the panel on
	/// demand).
	async fn dispose_output(&self, request:Request<DisposeOutputRequest>) -> Result<Response<Empty>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		let _ = self
			.environment
			.ApplicationHandle
			.emit("sky://output/dispose", json!({ "channel": req.channel_id }));
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

	/// extension info — look up a single extension by ID in ApplicationState
	async fn get_extension(
		&self,
		request:Request<GetExtensionRequest>,
	) -> Result<Response<GetExtensionResponse>, Status> {
		use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

		let req = request.into_inner();
		debug!("[CocoonService] get_extension: {}", req.extension_id);

		let ExtensionOption = self.environment.GetExtension(req.extension_id.clone()).await.ok().flatten();

		let InfoOption = ExtensionOption.map(|Value| {
			ExtensionInfo {
				id:req.extension_id,
				display_name:Value.get("Name").and_then(|V| V.as_str()).unwrap_or("").to_string(),
				version:Value.get("Version").and_then(|V| V.as_str()).unwrap_or("").to_string(),
				is_active:true, // scanned = considered active for now
				extension_path:Value
					.get("ExtensionLocation")
					.and_then(|V| V.as_str())
					.unwrap_or("")
					.to_string(),
			}
		});

		Ok(Response::new(GetExtensionResponse { extension:InfoOption }))
	}

	/// all extensions — return all scanned extensions from ApplicationState
	async fn get_all_extensions(&self, request:Request<Empty>) -> Result<Response<GetAllExtensionsResponse>, Status> {
		use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

		let _req = request.into_inner();

		let Extensions = self.environment.GetExtensions().await.unwrap_or_default();

		let ExtensionInfoList = Extensions
			.iter()
			.map(|Value| {
				ExtensionInfo {
					id:Value.get("Identifier").and_then(|V| V.as_str()).unwrap_or("").to_string(),
					display_name:Value.get("Name").and_then(|V| V.as_str()).unwrap_or("").to_string(),
					version:Value.get("Version").and_then(|V| V.as_str()).unwrap_or("").to_string(),
					is_active:true,
					extension_path:Value
						.get("ExtensionLocation")
						.and_then(|V| V.as_str())
						.unwrap_or("")
						.to_string(),
				}
			})
			.collect();

		Ok(Response::new(GetAllExtensionsResponse { extensions:ExtensionInfoList }))
	}

	/// Terminal Resize — emit a Tauri event so Sky can resize the xterm view.
	///
	/// PTY-level resize (via `portable_pty::MasterPty::resize`) is a P1 task
	/// that requires storing the PTY master handle in `TerminalStateDTO`.
	/// The Tauri event lets the UI immediately resize its canvas.
	async fn resize_terminal(&self, request:Request<ResizeTerminalRequest>) -> Result<Response<Empty>, Status> {
		use tauri::Emitter;

		let req = request.into_inner();
		debug!(
			"[CocoonService] resize_terminal: id={} cols={} rows={}",
			req.terminal_id, req.cols, req.rows
		);

		// Notify Sky/Wind of the new dimensions for UI resize
		let _ = self.environment.ApplicationHandle.emit(
			"sky://terminal/resize",
			json!({ "id": req.terminal_id, "cols": req.cols, "rows": req.rows }),
		);

		// TODO(P1): Call portable_pty::MasterPty::resize once PtyMaster handle
		// is stored in TerminalStateDTO (requires wrapping MasterPty in Arc<Mutex>)

		Ok(Response::new(Empty {}))
	}

	/// Get Configuration — retrieve a configuration value from
	/// ConfigurationProvider.
	async fn get_configuration(
		&self,
		request:Request<GetConfigurationRequest>,
	) -> Result<Response<GetConfigurationResponse>, Status> {
		use CommonLibrary::Configuration::{
			ConfigurationProvider::ConfigurationProvider,
			DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
		};

		let req = request.into_inner();
		let Key = if req.section.is_empty() {
			if req.key.is_empty() { None } else { Some(req.key.clone()) }
		} else if req.key.is_empty() {
			Some(req.section.clone())
		} else {
			Some(format!("{}.{}", req.section, req.key))
		};

		debug!("[CocoonService] get_configuration: key={:?}", Key);

		match self
			.environment
			.GetConfigurationValue(Key, ConfigurationOverridesDTO::default())
			.await
		{
			Ok(Value) => {
				let Bytes = serde_json::to_vec(&Value).unwrap_or_default();
				Ok(Response::new(GetConfigurationResponse { value:Bytes }))
			},
			Err(Error) => {
				warn!("[CocoonService] get_configuration failed: {}", Error);
				Ok(Response::new(GetConfigurationResponse::default()))
			},
		}
	}
}
