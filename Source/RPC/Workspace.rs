//! Workspace RPC service. `WorkspaceService::Struct` is the impl handle;
//! `WorkspaceFolder::Struct` and `TextDocumentInfo::Struct` are the DTOs
//! returned over the wire.
/// Text document info DTO: models an open document's URI, version, and language
/// ID.
pub mod TextDocumentInfo;

/// Workspace folder DTO: models a single workspace folder with URI and name.
pub mod WorkspaceFolder;

/// Workspace RPC service: routes file-and-workspace requests from the extension
/// host.
pub mod WorkspaceService;
