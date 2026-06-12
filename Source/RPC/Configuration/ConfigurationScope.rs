//! Scope of a configuration entry: Global / Workspace / Folder.
use serde::{Deserialize, Serialize};

/// Configuration scope: Global, Workspace, or Folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enum {
	Global,

	Workspace,

	Folder,
}
