//! Scope of a configuration entry: Global / Workspace / Folder.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enum {
	Global,

	Workspace,

	Folder,
}
