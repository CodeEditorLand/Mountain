//! Parsing and serialization of VS Code `.code-workspace` JSON files.
//!
//! The format is a JSON object with at minimum a `folders` array; each entry
//! has a `path` relative to the workspace file's parent directory.
//! `ParseWorkspaceFile::Fn` resolves each path through the canonical-path
//! cache and converts it to a `file://` URI.
//!
//! ## Sub-modules
//!
//! - [`ParseWorkspaceFile`]: Main entry point for workspace file parsing
//! - [`WorkspaceFile`]: Workspace file data model
//! - [`WorkspaceFolderEntry`]: Single workspace folder entry model

/// Main entry point for `.code-workspace` file parsing.
pub mod ParseWorkspaceFile;

pub(crate) mod WorkspaceFile;

pub(crate) mod WorkspaceFolderEntry;
