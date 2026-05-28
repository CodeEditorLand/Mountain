//! Type aliases that forward every `crate::Vine::Generated::*` reference to
//! the canonical `::Vine::Generated::*` types. Mountain's local `vine.rs`
//! was a byte-identical copy of Vine's generated file; this module replaces
//! it so all generated types are the same Rust types across the workspace.
//!
//! Adding a new protobuf message: add one `pub type Name = ::Vine::Generated::Name;` line.
//! Service modules are forwarded as inline `pub mod` blocks with type aliases.

#![allow(non_camel_case_types, unused_imports)]

// ─── Message types (alphabetical) ──────────────────────────────────────────

pub type AppendOutputRequest = ::Vine::Generated::AppendOutputRequest;
pub type ApplyEditRequest = ::Vine::Generated::ApplyEditRequest;
pub type ApplyEditResponse = ::Vine::Generated::ApplyEditResponse;
pub type Argument = ::Vine::Generated::Argument;
pub type CancelOperationRequest = ::Vine::Generated::CancelOperationRequest;
pub type ClearOutputRequest = ::Vine::Generated::ClearOutputRequest;
pub type CloseTerminalRequest = ::Vine::Generated::CloseTerminalRequest;
pub type CodeAction = ::Vine::Generated::CodeAction;
pub type CompletionItem = ::Vine::Generated::CompletionItem;
pub type CopyFileRequest = ::Vine::Generated::CopyFileRequest;
pub type CreateDirectoryRequest = ::Vine::Generated::CreateDirectoryRequest;
pub type CreateOutputChannelRequest = ::Vine::Generated::CreateOutputChannelRequest;
pub type CreateOutputChannelResponse = ::Vine::Generated::CreateOutputChannelResponse;
pub type CreateStatusBarItemRequest = ::Vine::Generated::CreateStatusBarItemRequest;
pub type CreateStatusBarItemResponse = ::Vine::Generated::CreateStatusBarItemResponse;
pub type CreateWebviewPanelRequest = ::Vine::Generated::CreateWebviewPanelRequest;
pub type CreateWebviewPanelResponse = ::Vine::Generated::CreateWebviewPanelResponse;
pub type DebugConfiguration = ::Vine::Generated::DebugConfiguration;
pub type DeleteFileRequest = ::Vine::Generated::DeleteFileRequest;
pub type DeleteSecretRequest = ::Vine::Generated::DeleteSecretRequest;
pub type DisposeOutputRequest = ::Vine::Generated::DisposeOutputRequest;
pub type DisposeWebviewPanelRequest = ::Vine::Generated::DisposeWebviewPanelRequest;
pub type Empty = ::Vine::Generated::Empty;
pub type Envelope = ::Vine::Generated::Envelope;
pub type ExecuteCommandRequest = ::Vine::Generated::ExecuteCommandRequest;
pub type ExecuteCommandResponse = ::Vine::Generated::ExecuteCommandResponse;
pub type ExecuteTaskRequest = ::Vine::Generated::ExecuteTaskRequest;
pub type ExecuteTaskResponse = ::Vine::Generated::ExecuteTaskResponse;
pub type ExtensionInfo = ::Vine::Generated::ExtensionInfo;
pub type FindFilesRequest = ::Vine::Generated::FindFilesRequest;
pub type FindFilesResponse = ::Vine::Generated::FindFilesResponse;
pub type FindTextInFilesRequest = ::Vine::Generated::FindTextInFilesRequest;
pub type FindTextInFilesResponse = ::Vine::Generated::FindTextInFilesResponse;
pub type GenericNotification = ::Vine::Generated::GenericNotification;
pub type GenericRequest = ::Vine::Generated::GenericRequest;
pub type GenericResponse = ::Vine::Generated::GenericResponse;
pub type GetAllExtensionsResponse = ::Vine::Generated::GetAllExtensionsResponse;
pub type GetAuthenticationSessionRequest = ::Vine::Generated::GetAuthenticationSessionRequest;
pub type GetAuthenticationSessionResponse = ::Vine::Generated::GetAuthenticationSessionResponse;
pub type GetConfigurationRequest = ::Vine::Generated::GetConfigurationRequest;
pub type GetConfigurationResponse = ::Vine::Generated::GetConfigurationResponse;
pub type GetExtensionRequest = ::Vine::Generated::GetExtensionRequest;
pub type GetExtensionResponse = ::Vine::Generated::GetExtensionResponse;
pub type GetSecretRequest = ::Vine::Generated::GetSecretRequest;
pub type GetSecretResponse = ::Vine::Generated::GetSecretResponse;
pub type GetTreeChildrenRequest = ::Vine::Generated::GetTreeChildrenRequest;
pub type GetTreeChildrenResponse = ::Vine::Generated::GetTreeChildrenResponse;
pub type GitExecRequest = ::Vine::Generated::GitExecRequest;
pub type GitExecResponse = ::Vine::Generated::GitExecResponse;
pub type InitExtensionHostRequest = ::Vine::Generated::InitExtensionHostRequest;
pub type Location = ::Vine::Generated::Location;
pub type OnDidReceiveMessageRequest = ::Vine::Generated::OnDidReceiveMessageRequest;
pub type OpenDocumentRequest = ::Vine::Generated::OpenDocumentRequest;
pub type OpenDocumentResponse = ::Vine::Generated::OpenDocumentResponse;
pub type OpenExternalRequest = ::Vine::Generated::OpenExternalRequest;
pub type OpenTerminalRequest = ::Vine::Generated::OpenTerminalRequest;
pub type ParticipateInSaveRequest = ::Vine::Generated::ParticipateInSaveRequest;
pub type ParticipateInSaveResponse = ::Vine::Generated::ParticipateInSaveResponse;
pub type Position = ::Vine::Generated::Position;
pub type PostWebviewMessageRequest = ::Vine::Generated::PostWebviewMessageRequest;
pub type ProvideCallHierarchyRequest = ::Vine::Generated::ProvideCallHierarchyRequest;
pub type ProvideCallHierarchyResponse = ::Vine::Generated::ProvideCallHierarchyResponse;
pub type ProvideCodeActionsRequest = ::Vine::Generated::ProvideCodeActionsRequest;
pub type ProvideCodeActionsResponse = ::Vine::Generated::ProvideCodeActionsResponse;
pub type ProvideCodeLensesRequest = ::Vine::Generated::ProvideCodeLensesRequest;
pub type ProvideCodeLensesResponse = ::Vine::Generated::ProvideCodeLensesResponse;
pub type ProvideCompletionItemsRequest = ::Vine::Generated::ProvideCompletionItemsRequest;
pub type ProvideCompletionItemsResponse = ::Vine::Generated::ProvideCompletionItemsResponse;
pub type ProvideDefinitionRequest = ::Vine::Generated::ProvideDefinitionRequest;
pub type ProvideDefinitionResponse = ::Vine::Generated::ProvideDefinitionResponse;
pub type ProvideDocumentFormattingRequest = ::Vine::Generated::ProvideDocumentFormattingRequest;
pub type ProvideDocumentFormattingResponse = ::Vine::Generated::ProvideDocumentFormattingResponse;
pub type ProvideDocumentHighlightsRequest = ::Vine::Generated::ProvideDocumentHighlightsRequest;
pub type ProvideDocumentHighlightsResponse = ::Vine::Generated::ProvideDocumentHighlightsResponse;
pub type ProvideDocumentRangeFormattingRequest =
	::Vine::Generated::ProvideDocumentRangeFormattingRequest;
pub type ProvideDocumentRangeFormattingResponse =
	::Vine::Generated::ProvideDocumentRangeFormattingResponse;
pub type ProvideDocumentSymbolsRequest = ::Vine::Generated::ProvideDocumentSymbolsRequest;
pub type ProvideDocumentSymbolsResponse = ::Vine::Generated::ProvideDocumentSymbolsResponse;
pub type ProvideFoldingRangesRequest = ::Vine::Generated::ProvideFoldingRangesRequest;
pub type ProvideFoldingRangesResponse = ::Vine::Generated::ProvideFoldingRangesResponse;
pub type ProvideHoverRequest = ::Vine::Generated::ProvideHoverRequest;
pub type ProvideHoverResponse = ::Vine::Generated::ProvideHoverResponse;
pub type ProvideInlayHintsRequest = ::Vine::Generated::ProvideInlayHintsRequest;
pub type ProvideInlayHintsResponse = ::Vine::Generated::ProvideInlayHintsResponse;
pub type ProvideInlineCompletionRequest = ::Vine::Generated::ProvideInlineCompletionRequest;
pub type ProvideInlineCompletionResponse = ::Vine::Generated::ProvideInlineCompletionResponse;
pub type ProvideLinkedEditingRangesRequest = ::Vine::Generated::ProvideLinkedEditingRangesRequest;
pub type ProvideLinkedEditingRangesResponse =
	::Vine::Generated::ProvideLinkedEditingRangesResponse;
pub type ProvideOnTypeFormattingRequest = ::Vine::Generated::ProvideOnTypeFormattingRequest;
pub type ProvideOnTypeFormattingResponse = ::Vine::Generated::ProvideOnTypeFormattingResponse;
pub type ProvideReferencesRequest = ::Vine::Generated::ProvideReferencesRequest;
pub type ProvideReferencesResponse = ::Vine::Generated::ProvideReferencesResponse;
pub type ProvideRenameEditsRequest = ::Vine::Generated::ProvideRenameEditsRequest;
pub type ProvideRenameEditsResponse = ::Vine::Generated::ProvideRenameEditsResponse;
pub type ProvideSelectionRangesRequest = ::Vine::Generated::ProvideSelectionRangesRequest;
pub type ProvideSelectionRangesResponse = ::Vine::Generated::ProvideSelectionRangesResponse;
pub type ProvideSemanticTokensRequest = ::Vine::Generated::ProvideSemanticTokensRequest;
pub type ProvideSemanticTokensResponse = ::Vine::Generated::ProvideSemanticTokensResponse;
pub type ProvideSignatureHelpRequest = ::Vine::Generated::ProvideSignatureHelpRequest;
pub type ProvideSignatureHelpResponse = ::Vine::Generated::ProvideSignatureHelpResponse;
pub type ProvideTypeHierarchyRequest = ::Vine::Generated::ProvideTypeHierarchyRequest;
pub type ProvideTypeHierarchyResponse = ::Vine::Generated::ProvideTypeHierarchyResponse;
pub type ProvideWorkspaceSymbolsRequest = ::Vine::Generated::ProvideWorkspaceSymbolsRequest;
pub type ProvideWorkspaceSymbolsResponse = ::Vine::Generated::ProvideWorkspaceSymbolsResponse;
pub type Range = ::Vine::Generated::Range;
pub type ReaddirRequest = ::Vine::Generated::ReaddirRequest;
pub type ReaddirResponse = ::Vine::Generated::ReaddirResponse;
pub type ReadFileRequest = ::Vine::Generated::ReadFileRequest;
pub type ReadFileResponse = ::Vine::Generated::ReadFileResponse;
pub type RegisterAuthenticationProviderRequest =
	::Vine::Generated::RegisterAuthenticationProviderRequest;
pub type RegisterCommandRequest = ::Vine::Generated::RegisterCommandRequest;
pub type RegisterDebugAdapterRequest = ::Vine::Generated::RegisterDebugAdapterRequest;
pub type RegisterOnTypeFormattingProviderRequest =
	::Vine::Generated::RegisterOnTypeFormattingProviderRequest;
pub type RegisterProviderRequest = ::Vine::Generated::RegisterProviderRequest;
pub type RegisterScmProviderRequest = ::Vine::Generated::RegisterScmProviderRequest;
pub type RegisterSemanticTokensProviderRequest =
	::Vine::Generated::RegisterSemanticTokensProviderRequest;
pub type RegisterSignatureHelpProviderRequest =
	::Vine::Generated::RegisterSignatureHelpProviderRequest;
pub type RegisterTaskProviderRequest = ::Vine::Generated::RegisterTaskProviderRequest;
pub type RegisterTreeViewProviderRequest = ::Vine::Generated::RegisterTreeViewProviderRequest;
pub type RenameFileRequest = ::Vine::Generated::RenameFileRequest;
pub type ReportProgressRequest = ::Vine::Generated::ReportProgressRequest;
pub type ResizeTerminalRequest = ::Vine::Generated::ResizeTerminalRequest;
pub type RpcError = ::Vine::Generated::RpcError;
pub type SaveAllRequest = ::Vine::Generated::SaveAllRequest;
pub type SaveAllResponse = ::Vine::Generated::SaveAllResponse;
pub type SetStatusBarTextRequest = ::Vine::Generated::SetStatusBarTextRequest;
pub type SetWebviewHtmlRequest = ::Vine::Generated::SetWebviewHtmlRequest;
pub type ShowInputBoxRequest = ::Vine::Generated::ShowInputBoxRequest;
pub type ShowInputBoxResponse = ::Vine::Generated::ShowInputBoxResponse;
pub type ShowMessageRequest = ::Vine::Generated::ShowMessageRequest;
pub type ShowMessageResponse = ::Vine::Generated::ShowMessageResponse;
pub type ShowOutputRequest = ::Vine::Generated::ShowOutputRequest;
pub type ShowProgressRequest = ::Vine::Generated::ShowProgressRequest;
pub type ShowProgressResponse = ::Vine::Generated::ShowProgressResponse;
pub type ShowQuickPickRequest = ::Vine::Generated::ShowQuickPickRequest;
pub type ShowQuickPickResponse = ::Vine::Generated::ShowQuickPickResponse;
pub type ShowTextDocumentRequest = ::Vine::Generated::ShowTextDocumentRequest;
pub type ShowTextDocumentResponse = ::Vine::Generated::ShowTextDocumentResponse;
pub type SourceControlResourceState = ::Vine::Generated::SourceControlResourceState;
pub type StartDebuggingRequest = ::Vine::Generated::StartDebuggingRequest;
pub type StartDebuggingResponse = ::Vine::Generated::StartDebuggingResponse;
pub type StatRequest = ::Vine::Generated::StatRequest;
pub type StatResponse = ::Vine::Generated::StatResponse;
pub type StopDebuggingRequest = ::Vine::Generated::StopDebuggingRequest;
pub type StoreSecretRequest = ::Vine::Generated::StoreSecretRequest;
pub type TerminalClosedNotification = ::Vine::Generated::TerminalClosedNotification;
pub type TerminalDataNotification = ::Vine::Generated::TerminalDataNotification;
pub type TerminalInputRequest = ::Vine::Generated::TerminalInputRequest;
pub type TerminalOpenedNotification = ::Vine::Generated::TerminalOpenedNotification;
pub type TerminalProcessIdNotification = ::Vine::Generated::TerminalProcessIdNotification;
pub type TerminateTaskRequest = ::Vine::Generated::TerminateTaskRequest;
pub type TextDocumentSaveReason = ::Vine::Generated::TextDocumentSaveReason;
pub type TextEdit = ::Vine::Generated::TextEdit;
pub type TextEditForSave = ::Vine::Generated::TextEditForSave;
pub type TextMatch = ::Vine::Generated::TextMatch;
pub type InlineCompletionItem = ::Vine::Generated::InlineCompletionItem;
pub type TreeItem = ::Vine::Generated::TreeItem;
pub type UnregisterCommandRequest = ::Vine::Generated::UnregisterCommandRequest;
pub type UpdateConfigurationRequest = ::Vine::Generated::UpdateConfigurationRequest;
pub type UpdateScmGroupRequest = ::Vine::Generated::UpdateScmGroupRequest;
pub type UpdateWorkspaceFoldersRequest = ::Vine::Generated::UpdateWorkspaceFoldersRequest;
pub type Uri = ::Vine::Generated::Uri;
pub type ViewColumn = ::Vine::Generated::ViewColumn;
pub type WatchFileRequest = ::Vine::Generated::WatchFileRequest;
pub type WorkspaceFolder = ::Vine::Generated::WorkspaceFolder;
pub type WriteFileRequest = ::Vine::Generated::WriteFileRequest;

// ─── oneof field sub-modules ────────────────────────────────────────────────

pub mod on_did_receive_message_request {
	pub type Message = ::Vine::Generated::on_did_receive_message_request::Message;
}

pub mod post_webview_message_request {
	pub type Message = ::Vine::Generated::post_webview_message_request::Message;
}

pub mod argument {
	pub type Value = ::Vine::Generated::argument::Value;
}

pub mod execute_command_response {
	pub type Result = ::Vine::Generated::execute_command_response::Result;
}

// ─── Service modules ────────────────────────────────────────────────────────

// Service module forwarding.
// Traits (MountainService, CocoonService) cannot be aliased in stable Rust -
// impl files use ::Vine::Generated::*::Trait directly. Only concrete struct
// types (servers, clients) are aliased here.
pub mod mountain_service_server {
	pub type MountainServiceServer<T> =
		::Vine::Generated::mountain_service_server::MountainServiceServer<T>;
}

pub mod cocoon_service_server {
	pub type CocoonServiceServer<T> =
		::Vine::Generated::cocoon_service_server::CocoonServiceServer<T>;
}

pub mod cocoon_service_client {
	pub type CocoonServiceClient<T> =
		::Vine::Generated::cocoon_service_client::CocoonServiceClient<T>;
}
