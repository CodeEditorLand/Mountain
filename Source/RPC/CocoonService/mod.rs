#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

// # CocoonServiceImpl - thin-wrapper dispatcher
//
// Domain files hold all typed RPC implementations. This module keeps:
// - CocoonServiceImpl struct + helper methods
// - process_mountain_request (legacy generic router, ~600 lines)
// - send_mountain_notification (push dispatcher, ~400 lines)
// - One-line delegates for all 78 typed RPCs

pub mod Auth;
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

#[allow(unused_imports)]
use std::{
	collections::HashMap,
	sync::Arc,
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

use crate::{
	ApplicationState::DTO::{
		ProviderRegistrationDTO::ProviderRegistrationDTO,
		WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	},
	Environment::MountainEnvironment::MountainEnvironment,
};
// Import generated protobuf types
use crate::dev_log;
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
	on_did_receive_message_request,
	post_webview_message_request,
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

		dev_log!(
			"cocoon",
			"[CocoonService] generic request: method={} id={}",
			Req.method,
			RequestId
		);

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
				let Options = OpenDialogOptionsDTO {
					Base:CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO {
						Title,
						..Default::default()
					},
					..OpenDialogOptionsDTO::default()
				};
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
				let Options = SaveDialogOptionsDTO {
					Base:CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO {
						Title,
						..Default::default()
					},
					..SaveDialogOptionsDTO::default()
				};
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
				// Emit to Sky - Sky uses Tauri shell plugin to open the URL
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
				let Items:Option<serde_json::Value> = Params
					.get("items")
					.cloned()
					.filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());
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
				let Items:Option<serde_json::Value> = Params
					.get("items")
					.cloned()
					.filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());
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
				let Items:Option<serde_json::Value> = Params
					.get("items")
					.cloned()
					.filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());
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
				// Canonical kebab-case channel; `sky://webview/setHtml` retired.
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://webview/set-html", json!({ "handle": Handle, "html": Html }));
				Ok(OkResponse(RequestId, &json!({ "success": true })))
			},
			// ---- Workspace (Cocoon MountainGRPCClient format) ----
			// `findFiles` / `findTextInFiles` are called by Cocoon's
			// `workspace.findFiles()` / `workspace.findTextInFiles()`
			// API shims. Delegate to the real trait implementations
			// (`WorkspaceProvider::FindFilesInWorkspace`,
			// `SearchProvider::TextSearch`) which use `ignore::WalkBuilder`
			// + `grep-searcher` - respecting `.gitignore`, doing parallel
			// walks, and producing properly-constructed `Url` results.
			// Prior inline implementations used naive dir-walks, hidden-
			// dot skipping, and `format!("file://{}", path)` URI
			// construction that mangled non-ASCII paths.
			"findFiles" => {
				use CommonLibrary::Workspace::WorkspaceProvider::WorkspaceProvider;
				let Include = Params
					.get("pattern")
					.cloned()
					.or_else(|| Params.get("include").cloned())
					.unwrap_or(serde_json::Value::String("**".into()));
				let Exclude = Params
					.get("exclude")
					.cloned()
					.filter(|V| !V.is_null());
				let MaxResults = Params
					.get("maxResults")
					.and_then(|V| V.as_u64())
					.map(|N| N as usize);
				let UseIgnoreFiles = Params
					.get("useIgnoreFiles")
					.and_then(|V| V.as_bool())
					.unwrap_or(true);
				let FollowSymlinks = Params
					.get("followSymlinks")
					.and_then(|V| V.as_bool())
					.unwrap_or(false);
				match self
					.environment
					.FindFilesInWorkspace(Include, Exclude, MaxResults, UseIgnoreFiles, FollowSymlinks)
					.await
				{
					Ok(Urls) => Ok(OkResponse(
						RequestId,
						&json!({ "uris": Urls.into_iter().map(|U| U.to_string()).collect::<Vec<_>>() }),
					)),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("findFiles: {}", Error))),
				}
			},
			"findTextInFiles" => {
				use CommonLibrary::Search::SearchProvider::SearchProvider;
				// VS Code's `workspace.findTextInFiles` takes a
				// `TextSearchQuery` in field `pattern` (or passed flat
				// at the top level). Accept both shapes.
				let QueryValue = if Params.get("pattern").map(|V| V.is_object()).unwrap_or(false) {
					Params.get("pattern").cloned().unwrap_or(serde_json::Value::Null)
				} else if Params.get("pattern").map(|V| V.is_string()).unwrap_or(false) {
					json!({
						"pattern": Params.get("pattern").and_then(|V| V.as_str()).unwrap_or(""),
						"isRegExp": Params.get("isRegExp").and_then(|V| V.as_bool()).unwrap_or(false),
						"isCaseSensitive": Params.get("isCaseSensitive").and_then(|V| V.as_bool()).unwrap_or(false),
						"isWordMatch": Params.get("isWordMatch").and_then(|V| V.as_bool()).unwrap_or(false),
					})
				} else {
					Params.clone()
				};
				let OptionsValue = Params
					.get("options")
					.cloned()
					.unwrap_or(serde_json::Value::Null);
				match self.environment.TextSearch(QueryValue, OptionsValue).await {
					Ok(Matches) => Ok(OkResponse(RequestId, &json!({ "matches": Matches }))),
					Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("findTextInFiles: {}", Error))),
				}
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
				dev_log!("cocoon", "warn: [CocoonService] Unknown generic method: {}", Req.method);
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
		dev_log!(
			"cocoon",
			"[CocoonService] Notification router: method='{}'",
			notification.method
		);

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
					dev_log!(
						"cocoon",
						"warn: [CocoonService] notification: registerCommand '{}' failed: {:?}",
						CommandId,
						Error
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
				let _ = self.environment.ApplicationHandle.emit(
					"sky://webview/message",
					json!({ "panelId": PanelId, "method": Method, "params": MsgParams }),
				);
			},
			"webview.dispose" => {
				use tauri::Emitter;
				let PanelId = Params.get("panelId").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://webview/dispose", json!({ "panelId": PanelId }));
			},
			// ---- Progress indicator ----
			"progress.start" => {
				use tauri::Emitter;
				let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Title = Params.get("title").and_then(|V| V.as_str()).map(|S| S.to_string());
				let Location = Params.get("location").cloned();
				let Cancellable = Params.get("cancellable").and_then(|V| V.as_bool()).unwrap_or(false);
				let _ = self.environment.ApplicationHandle.emit(
					"sky://progress/start",
					json!({ "id": Id, "title": Title, "location": Location, "cancellable": Cancellable }),
				);
			},
			"progress.update" => {
				use tauri::Emitter;
				let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let Message = Params.get("message").and_then(|V| V.as_str()).map(|S| S.to_string());
				let Increment = Params.get("increment").and_then(|V| V.as_f64());
				let _ = self.environment.ApplicationHandle.emit(
					"sky://progress/update",
					json!({ "id": Id, "message": Message, "increment": Increment }),
				);
			},
			"progress.complete" => {
				use tauri::Emitter;
				let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://progress/complete", json!({ "id": Id }));
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
				// Language configuration is consumed by Sky - emit for workbench to pick up
				use tauri::Emitter;
				let Language = Params.get("language").and_then(|V| V.as_str()).unwrap_or("").to_string();
				let _ = self
					.environment
					.ApplicationHandle
					.emit("sky://language/configure", json!({ "language": Language }));
			},
			_ => {
				dev_log!(
					"cocoon",
					"[CocoonService] Unknown notification method: '{}'",
					notification.method
				);
			},
		}

		Ok(Response::new(Empty {}))
	}

	/// Cancel operations requested by Mountain

	async fn cancel_operation(&self, request:Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		Initialization::CancelOperation(self, request.into_inner()).await
	}

	async fn initial_handshake(&self, request:Request<Empty>) -> Result<Response<Empty>, Status> {
		Initialization::InitialHandshake(self, request.into_inner()).await
	}

	async fn init_extension_host(&self, request:Request<InitExtensionHostRequest>) -> Result<Response<Empty>, Status> {
		Initialization::InitExtensionHost(self, request.into_inner()).await
	}

	async fn register_command(&self, request:Request<RegisterCommandRequest>) -> Result<Response<Empty>, Status> {
		Command::RegisterCommand(self, request.into_inner()).await
	}

	async fn execute_contributed_command(&self, request:Request<ExecuteCommandRequest>) -> Result<Response<ExecuteCommandResponse>, Status> {
		Command::ExecuteContributedCommand(self, request.into_inner()).await
	}

	async fn unregister_command(&self, request:Request<UnregisterCommandRequest>) -> Result<Response<Empty>, Status> {
		Command::UnregisterCommand(self, request.into_inner()).await
	}

	async fn register_hover_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterHoverProvider(self, request.into_inner()).await
	}

	async fn provide_hover(&self, request:Request<ProvideHoverRequest>) -> Result<Response<ProvideHoverResponse>, Status> {
		Provider::ProvideHover(self, request.into_inner()).await
	}

	async fn register_completion_item_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterCompletionItemProvider(self, request.into_inner()).await
	}

	async fn provide_completion_items(&self, request:Request<ProvideCompletionItemsRequest>) -> Result<Response<ProvideCompletionItemsResponse>, Status> {
		Provider::ProvideCompletionItems(self, request.into_inner()).await
	}

	async fn register_definition_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterDefinitionProvider(self, request.into_inner()).await
	}

	async fn provide_definition(&self, request:Request<ProvideDefinitionRequest>) -> Result<Response<ProvideDefinitionResponse>, Status> {
		Provider::ProvideDefinition(self, request.into_inner()).await
	}

	async fn register_reference_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterReferenceProvider(self, request.into_inner()).await
	}

	async fn provide_references(&self, request:Request<ProvideReferencesRequest>) -> Result<Response<ProvideReferencesResponse>, Status> {
		Provider::ProvideReferences(self, request.into_inner()).await
	}

	async fn register_code_actions_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterCodeActionsProvider(self, request.into_inner()).await
	}

	async fn provide_code_actions(&self, request:Request<ProvideCodeActionsRequest>) -> Result<Response<ProvideCodeActionsResponse>, Status> {
		Provider::ProvideCodeActions(self, request.into_inner()).await
	}

	async fn show_text_document(&self, request:Request<ShowTextDocumentRequest>) -> Result<Response<ShowTextDocumentResponse>, Status> {
		Window::ShowTextDocument(self, request.into_inner()).await
	}

	async fn show_information_message(&self, request:Request<ShowMessageRequest>) -> Result<Response<ShowMessageResponse>, Status> {
		Window::ShowInformationMessage(self, request.into_inner()).await
	}

	async fn show_warning_message(&self, request:Request<ShowMessageRequest>) -> Result<Response<ShowMessageResponse>, Status> {
		Window::ShowWarningMessage(self, request.into_inner()).await
	}

	async fn show_error_message(&self, request:Request<ShowMessageRequest>) -> Result<Response<ShowMessageResponse>, Status> {
		Window::ShowErrorMessage(self, request.into_inner()).await
	}

	async fn create_status_bar_item(&self, request:Request<CreateStatusBarItemRequest>) -> Result<Response<CreateStatusBarItemResponse>, Status> {
		Window::CreateStatusBarItem(self, request.into_inner()).await
	}

	async fn set_status_bar_text(&self, request:Request<SetStatusBarTextRequest>) -> Result<Response<Empty>, Status> {
		Window::SetStatusBarText(self, request.into_inner()).await
	}

	async fn create_webview_panel(&self, request:Request<CreateWebviewPanelRequest>) -> Result<Response<CreateWebviewPanelResponse>, Status> {
		Window::CreateWebviewPanel(self, request.into_inner()).await
	}

	async fn set_webview_html(&self, request:Request<SetWebviewHtmlRequest>) -> Result<Response<Empty>, Status> {
		Window::SetWebviewHtml(self, request.into_inner()).await
	}

	async fn on_did_receive_message(&self, request:Request<OnDidReceiveMessageRequest>) -> Result<Response<Empty>, Status> {
		Window::OnDidReceiveMessage(self, request.into_inner()).await
	}

	async fn post_webview_message(&self, request:Request<PostWebviewMessageRequest>) -> Result<Response<Empty>, Status> {
		Window::PostWebviewMessage(self, request.into_inner()).await
	}

	async fn dispose_webview_panel(&self, request:Request<DisposeWebviewPanelRequest>) -> Result<Response<Empty>, Status> {
		Window::DisposeWebviewPanel(self, request.into_inner()).await
	}

	async fn read_file(&self, request:Request<ReadFileRequest>) -> Result<Response<ReadFileResponse>, Status> {
		FileSystem::ReadFile(self, request.into_inner()).await
	}

	async fn write_file(&self, request:Request<WriteFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::WriteFile(self, request.into_inner()).await
	}

	async fn stat(&self, request:Request<StatRequest>) -> Result<Response<StatResponse>, Status> {
		FileSystem::Stat(self, request.into_inner()).await
	}

	async fn readdir(&self, request:Request<ReaddirRequest>) -> Result<Response<ReaddirResponse>, Status> {
		FileSystem::Readdir(self, request.into_inner()).await
	}

	async fn watch_file(&self, request:Request<WatchFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::WatchFile(self, request.into_inner()).await
	}

	async fn find_files(&self, request:Request<FindFilesRequest>) -> Result<Response<FindFilesResponse>, Status> {
		FileSystem::FindFiles(self, request.into_inner()).await
	}

	async fn find_text_in_files(&self, request:Request<FindTextInFilesRequest>) -> Result<Response<FindTextInFilesResponse>, Status> {
		FileSystem::FindTextInFiles(self, request.into_inner()).await
	}

	async fn delete_file(&self, request:Request<DeleteFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::DeleteFile(self, request.into_inner()).await
	}

	async fn rename_file(&self, request:Request<RenameFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::RenameFile(self, request.into_inner()).await
	}

	async fn copy_file(&self, request:Request<CopyFileRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::CopyFile(self, request.into_inner()).await
	}

	async fn create_directory(&self, request:Request<CreateDirectoryRequest>) -> Result<Response<Empty>, Status> {
		FileSystem::CreateDirectory(self, request.into_inner()).await
	}

	async fn open_document(&self, request:Request<OpenDocumentRequest>) -> Result<Response<OpenDocumentResponse>, Status> {
		Workspace::OpenDocument(self, request.into_inner()).await
	}

	async fn save_all(&self, request:Request<SaveAllRequest>) -> Result<Response<SaveAllResponse>, Status> {
		Workspace::SaveAll(self, request.into_inner()).await
	}

	async fn apply_edit(&self, request:Request<ApplyEditRequest>) -> Result<Response<ApplyEditResponse>, Status> {
		Workspace::ApplyEdit(self, request.into_inner()).await
	}

	async fn update_configuration(&self, request:Request<UpdateConfigurationRequest>) -> Result<Response<Empty>, Status> {
		Workspace::UpdateConfiguration(self, request.into_inner()).await
	}

	async fn update_workspace_folders(&self, request:Request<UpdateWorkspaceFoldersRequest>) -> Result<Response<Empty>, Status> {
		Workspace::UpdateWorkspaceFolders(self, request.into_inner()).await
	}

	async fn open_terminal(&self, request:Request<OpenTerminalRequest>) -> Result<Response<Empty>, Status> {
		Terminal::OpenTerminal(self, request.into_inner()).await
	}

	async fn terminal_input(&self, request:Request<TerminalInputRequest>) -> Result<Response<Empty>, Status> {
		Terminal::TerminalInput(self, request.into_inner()).await
	}

	async fn close_terminal(&self, request:Request<CloseTerminalRequest>) -> Result<Response<Empty>, Status> {
		Terminal::CloseTerminal(self, request.into_inner()).await
	}

	async fn accept_terminal_opened(&self, request:Request<TerminalOpenedNotification>) -> Result<Response<Empty>, Status> {
		Terminal::AcceptTerminalOpened(self, request.into_inner()).await
	}

	async fn accept_terminal_closed(&self, request:Request<TerminalClosedNotification>) -> Result<Response<Empty>, Status> {
		Terminal::AcceptTerminalClosed(self, request.into_inner()).await
	}

	async fn accept_terminal_process_id(&self, request:Request<TerminalProcessIdNotification>) -> Result<Response<Empty>, Status> {
		Terminal::AcceptTerminalProcessId(self, request.into_inner()).await
	}

	async fn accept_terminal_process_data(&self, request:Request<TerminalDataNotification>) -> Result<Response<Empty>, Status> {
		Terminal::AcceptTerminalProcessData(self, request.into_inner()).await
	}

	async fn resize_terminal(&self, request:Request<ResizeTerminalRequest>) -> Result<Response<Empty>, Status> {
		Terminal::ResizeTerminal(self, request.into_inner()).await
	}

	async fn register_tree_view_provider(&self, request:Request<RegisterTreeViewProviderRequest>) -> Result<Response<Empty>, Status> {
		TreeView::RegisterTreeViewProvider(self, request.into_inner()).await
	}

	async fn get_tree_children(&self, request:Request<GetTreeChildrenRequest>) -> Result<Response<GetTreeChildrenResponse>, Status> {
		TreeView::GetTreeChildren(self, request.into_inner()).await
	}

	async fn register_scm_provider(&self, request:Request<RegisterScmProviderRequest>) -> Result<Response<Empty>, Status> {
		SCM::RegisterScmProvider(self, request.into_inner()).await
	}

	async fn update_scm_group(&self, request:Request<UpdateScmGroupRequest>) -> Result<Response<Empty>, Status> {
		SCM::UpdateScmGroup(self, request.into_inner()).await
	}

	async fn git_exec(&self, request:Request<GitExecRequest>) -> Result<Response<GitExecResponse>, Status> {
		SCM::GitExec(self, request.into_inner()).await
	}

	async fn register_debug_adapter(&self, request:Request<RegisterDebugAdapterRequest>) -> Result<Response<Empty>, Status> {
		Debug::RegisterDebugAdapter(self, request.into_inner()).await
	}

	async fn start_debugging(&self, request:Request<StartDebuggingRequest>) -> Result<Response<StartDebuggingResponse>, Status> {
		Debug::StartDebugging(self, request.into_inner()).await
	}

	async fn stop_debugging(&self, request:Request<StopDebuggingRequest>) -> Result<Response<Empty>, Status> {
		Debug::StopDebugging(self, request.into_inner()).await
	}

	async fn participate_in_save(&self, request:Request<ParticipateInSaveRequest>) -> Result<Response<ParticipateInSaveResponse>, Status> {
		Save::ParticipateInSave(self, request.into_inner()).await
	}

	async fn get_secret(&self, request:Request<GetSecretRequest>) -> Result<Response<GetSecretResponse>, Status> {
		Secret::GetSecret(self, request.into_inner()).await
	}

	async fn store_secret(&self, request:Request<StoreSecretRequest>) -> Result<Response<Empty>, Status> {
		Secret::StoreSecret(self, request.into_inner()).await
	}

	async fn delete_secret(&self, request:Request<DeleteSecretRequest>) -> Result<Response<Empty>, Status> {
		Secret::DeleteSecret(self, request.into_inner()).await
	}

	async fn register_document_highlight_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterDocumentHighlightProvider(self, request.into_inner()).await
	}

	async fn provide_document_highlights(&self, request:Request<ProvideDocumentHighlightsRequest>) -> Result<Response<ProvideDocumentHighlightsResponse>, Status> {
		Provider::ProvideDocumentHighlights(self, request.into_inner()).await
	}

	async fn register_document_symbol_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterDocumentSymbolProvider(self, request.into_inner()).await
	}

	async fn provide_document_symbols(&self, request:Request<ProvideDocumentSymbolsRequest>) -> Result<Response<ProvideDocumentSymbolsResponse>, Status> {
		Provider::ProvideDocumentSymbols(self, request.into_inner()).await
	}

	async fn register_workspace_symbol_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterWorkspaceSymbolProvider(self, request.into_inner()).await
	}

	async fn provide_workspace_symbols(&self, request:Request<ProvideWorkspaceSymbolsRequest>) -> Result<Response<ProvideWorkspaceSymbolsResponse>, Status> {
		Provider::ProvideWorkspaceSymbols(self, request.into_inner()).await
	}

	async fn register_rename_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterRenameProvider(self, request.into_inner()).await
	}

	async fn provide_rename_edits(&self, request:Request<ProvideRenameEditsRequest>) -> Result<Response<ProvideRenameEditsResponse>, Status> {
		Provider::ProvideRenameEdits(self, request.into_inner()).await
	}

	async fn register_document_formatting_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterDocumentFormattingProvider(self, request.into_inner()).await
	}

	async fn provide_document_formatting(&self, request:Request<ProvideDocumentFormattingRequest>) -> Result<Response<ProvideDocumentFormattingResponse>, Status> {
		Provider::ProvideDocumentFormatting(self, request.into_inner()).await
	}

	async fn register_document_range_formatting_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterDocumentRangeFormattingProvider(self, request.into_inner()).await
	}

	async fn provide_document_range_formatting(&self, request:Request<ProvideDocumentRangeFormattingRequest>) -> Result<Response<ProvideDocumentRangeFormattingResponse>, Status> {
		Provider::ProvideDocumentRangeFormatting(self, request.into_inner()).await
	}

	async fn register_on_type_formatting_provider(&self, request:Request<RegisterOnTypeFormattingProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterOnTypeFormattingProvider(self, request.into_inner()).await
	}

	async fn provide_on_type_formatting(&self, request:Request<ProvideOnTypeFormattingRequest>) -> Result<Response<ProvideOnTypeFormattingResponse>, Status> {
		Provider::ProvideOnTypeFormatting(self, request.into_inner()).await
	}

	async fn register_signature_help_provider(&self, request:Request<RegisterSignatureHelpProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterSignatureHelpProvider(self, request.into_inner()).await
	}

	async fn provide_signature_help(&self, request:Request<ProvideSignatureHelpRequest>) -> Result<Response<ProvideSignatureHelpResponse>, Status> {
		Provider::ProvideSignatureHelp(self, request.into_inner()).await
	}

	async fn register_code_lens_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterCodeLensProvider(self, request.into_inner()).await
	}

	async fn provide_code_lenses(&self, request:Request<ProvideCodeLensesRequest>) -> Result<Response<ProvideCodeLensesResponse>, Status> {
		Provider::ProvideCodeLenses(self, request.into_inner()).await
	}

	async fn register_folding_range_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterFoldingRangeProvider(self, request.into_inner()).await
	}

	async fn provide_folding_ranges(&self, request:Request<ProvideFoldingRangesRequest>) -> Result<Response<ProvideFoldingRangesResponse>, Status> {
		Provider::ProvideFoldingRanges(self, request.into_inner()).await
	}

	async fn register_selection_range_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterSelectionRangeProvider(self, request.into_inner()).await
	}

	async fn provide_selection_ranges(&self, request:Request<ProvideSelectionRangesRequest>) -> Result<Response<ProvideSelectionRangesResponse>, Status> {
		Provider::ProvideSelectionRanges(self, request.into_inner()).await
	}

	async fn register_semantic_tokens_provider(&self, request:Request<RegisterSemanticTokensProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterSemanticTokensProvider(self, request.into_inner()).await
	}

	async fn provide_semantic_tokens_full(&self, request:Request<ProvideSemanticTokensRequest>) -> Result<Response<ProvideSemanticTokensResponse>, Status> {
		Provider::ProvideSemanticTokensFull(self, request.into_inner()).await
	}

	async fn register_inlay_hints_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterInlayHintsProvider(self, request.into_inner()).await
	}

	async fn provide_inlay_hints(&self, request:Request<ProvideInlayHintsRequest>) -> Result<Response<ProvideInlayHintsResponse>, Status> {
		Provider::ProvideInlayHints(self, request.into_inner()).await
	}

	async fn register_type_hierarchy_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterTypeHierarchyProvider(self, request.into_inner()).await
	}

	async fn provide_type_hierarchy_supertypes(&self, request:Request<ProvideTypeHierarchyRequest>) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
		Provider::ProvideTypeHierarchySupertypes(self, request.into_inner()).await
	}

	async fn provide_type_hierarchy_subtypes(&self, request:Request<ProvideTypeHierarchyRequest>) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
		Provider::ProvideTypeHierarchySubtypes(self, request.into_inner()).await
	}

	async fn register_call_hierarchy_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterCallHierarchyProvider(self, request.into_inner()).await
	}

	async fn provide_call_hierarchy_incoming_calls(&self, request:Request<ProvideCallHierarchyRequest>) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
		Provider::ProvideCallHierarchyIncomingCalls(self, request.into_inner()).await
	}

	async fn provide_call_hierarchy_outgoing_calls(&self, request:Request<ProvideCallHierarchyRequest>) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
		Provider::ProvideCallHierarchyOutgoingCalls(self, request.into_inner()).await
	}

	async fn register_linked_editing_range_provider(&self, request:Request<RegisterProviderRequest>) -> Result<Response<Empty>, Status> {
		Provider::RegisterLinkedEditingRangeProvider(self, request.into_inner()).await
	}

	async fn provide_linked_editing_ranges(&self, request:Request<ProvideLinkedEditingRangesRequest>) -> Result<Response<ProvideLinkedEditingRangesResponse>, Status> {
		Provider::ProvideLinkedEditingRanges(self, request.into_inner()).await
	}

	async fn show_quick_pick(&self, request:Request<ShowQuickPickRequest>) -> Result<Response<ShowQuickPickResponse>, Status> {
		Window::ShowQuickPick(self, request.into_inner()).await
	}

	async fn show_input_box(&self, request:Request<ShowInputBoxRequest>) -> Result<Response<ShowInputBoxResponse>, Status> {
		Window::ShowInputBox(self, request.into_inner()).await
	}

	async fn show_progress(&self, request:Request<ShowProgressRequest>) -> Result<Response<ShowProgressResponse>, Status> {
		Window::ShowProgress(self, request.into_inner()).await
	}

	async fn report_progress(&self, request:Request<ReportProgressRequest>) -> Result<Response<Empty>, Status> {
		Window::ReportProgress(self, request.into_inner()).await
	}

	async fn open_external(&self, request:Request<OpenExternalRequest>) -> Result<Response<Empty>, Status> {
		Window::OpenExternal(self, request.into_inner()).await
	}

	async fn create_output_channel(&self, request:Request<CreateOutputChannelRequest>) -> Result<Response<CreateOutputChannelResponse>, Status> {
		Output::CreateOutputChannel(self, request.into_inner()).await
	}

	async fn append_output(&self, request:Request<AppendOutputRequest>) -> Result<Response<Empty>, Status> {
		Output::AppendOutput(self, request.into_inner()).await
	}

	async fn clear_output(&self, request:Request<ClearOutputRequest>) -> Result<Response<Empty>, Status> {
		Output::ClearOutput(self, request.into_inner()).await
	}

	async fn show_output(&self, request:Request<ShowOutputRequest>) -> Result<Response<Empty>, Status> {
		Output::ShowOutput(self, request.into_inner()).await
	}

	async fn dispose_output(&self, request:Request<DisposeOutputRequest>) -> Result<Response<Empty>, Status> {
		Output::DisposeOutput(self, request.into_inner()).await
	}

	async fn register_task_provider(&self, request:Request<RegisterTaskProviderRequest>) -> Result<Response<Empty>, Status> {
		Task::RegisterTaskProvider(self, request.into_inner()).await
	}

	async fn execute_task(&self, request:Request<ExecuteTaskRequest>) -> Result<Response<ExecuteTaskResponse>, Status> {
		Task::ExecuteTask(self, request.into_inner()).await
	}

	async fn terminate_task(&self, request:Request<TerminateTaskRequest>) -> Result<Response<Empty>, Status> {
		Task::TerminateTask(self, request.into_inner()).await
	}

	async fn get_authentication_session(&self, request:Request<GetAuthenticationSessionRequest>) -> Result<Response<GetAuthenticationSessionResponse>, Status> {
		Auth::GetAuthenticationSession(self, request.into_inner()).await
	}

	async fn register_authentication_provider(&self, request:Request<RegisterAuthenticationProviderRequest>) -> Result<Response<Empty>, Status> {
		Auth::RegisterAuthenticationProvider(self, request.into_inner()).await
	}

	async fn get_extension(&self, request:Request<GetExtensionRequest>) -> Result<Response<GetExtensionResponse>, Status> {
		Extension::GetExtension(self, request.into_inner()).await
	}

	async fn get_all_extensions(&self, request:Request<Empty>) -> Result<Response<GetAllExtensionsResponse>, Status> {
		Extension::GetAllExtensions(self, request.into_inner()).await
	}

	async fn get_configuration(&self, request:Request<GetConfigurationRequest>) -> Result<Response<GetConfigurationResponse>, Status> {
		Extension::GetConfiguration(self, request.into_inner()).await
	}
}
