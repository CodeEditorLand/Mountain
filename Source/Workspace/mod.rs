#![allow(non_snake_case)]

//! Workspace lifecycle: `.code-workspace` parsing, multi-root folder
//! resolution, workspace-scoped configuration. Implements
//! `CommonLibrary::Workspace::WorkspaceProvider` over `ApplicationState`.

pub mod WorkspaceFileService;
