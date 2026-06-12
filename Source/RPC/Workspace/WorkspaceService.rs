//! File-and-workspace RPC service.
use std::path::PathBuf;

/// File-and-workspace RPC service handle.
pub struct Struct {
	workspace_root:Option<PathBuf>,
}

/// Creates a new `Struct` with no workspace root.
impl Struct {
	pub fn new() -> Self { Self { workspace_root:None } }

	pub fn with_root(Root:PathBuf) -> Self { Self { workspace_root:Some(Root) } }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
