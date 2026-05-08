#![allow(non_snake_case)]

//! Workspace RPC service. `WorkspaceService::Struct` is the impl handle;
//! `WorkspaceFolder::Struct` and `TextDocumentInfo::Struct` are the DTOs
//! returned over the wire.

pub mod TextDocumentInfo;

pub mod WorkspaceFolder;

pub mod WorkspaceService;
