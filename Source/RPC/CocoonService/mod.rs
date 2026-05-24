pub mod UnregisterOperation;
pub mod RegisterOperation;
pub mod new;
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

	cocoon_service_server::CocoonService,
	on_did_receive_message_request,
	post_webview_message_request,
};

/// Implementation of the CocoonService gRPC server
/// This struct handles all incoming requests from the Cocoon extension host
/// sidecar and dispatches them to the appropriate Mountain services.
#[derive(Clone)]
pub struct CocoonServiceImpl {
	/// Mountain environment providing access to all services
	environment:Arc<MountainEnvironment>,

	/// Registry of active operations with their cancellation tokens
	/// Maps request ID to cancellation token for operation cancellation
	ActiveOperations:Arc<RwLock<HashMap<u64, tokio_util::sync::CancellationToken>>>,

#[async_trait]
}

#[derive(Debug, Clone)]
pub struct Struct;
