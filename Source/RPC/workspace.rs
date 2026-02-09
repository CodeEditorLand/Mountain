//! # Workspace RPC Service
//!
//! Workspace service for file and workspace operations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Workspace service
pub struct WorkspaceService {
    workspace_root: Option<PathBuf>,
}

impl WorkspaceService {
    pub fn new() -> Self {
        Self {
            workspace_root: None,
        }
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self {
            workspace_root: Some(root),
        }
    }
}

impl Default for WorkspaceService {
    fn default() -> Self {
        Self::new()
    }
}

/// Workspace folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
}

/// Text document info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentInfo {
    pub uri: String,
    pub version: i32,
    pub language_id: String,
}
