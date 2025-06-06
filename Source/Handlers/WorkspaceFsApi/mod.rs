// File: Handlers/WorkspaceFsApi/mod.rs
// This module defines and exports handlers that implement the
// `vscode.workspace.fs` API. These handlers receive requests from the sidecar
// and use the application's runtime and environment to perform filesystem
// operations.

#![allow(non_snake_case, non_camel_case_types)]

mod WorkspaceFsApi; // Contains the logic for the workspace filesystem API handlers

pub use self::WorkspaceFsApi::*; // Re-export all public functions from WorkspaceFsApi.rs
