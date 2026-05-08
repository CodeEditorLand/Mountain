#![allow(non_snake_case)]

//! Parsing and serialization of VS Code `.code-workspace` JSON files.
//!
//! The format is a JSON object with at minimum a `folders` array; each entry
//! has a `path` relative to the workspace file's parent directory.
//! `ParseWorkspaceFile::Fn` resolves each path through the canonical-path
//! cache and converts it to a `file://` URI.

pub mod ParseWorkspaceFile;

pub(crate) mod WorkspaceFile;

pub(crate) mod WorkspaceFolderEntry;
