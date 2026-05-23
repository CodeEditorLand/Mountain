
//! File-and-workspace RPC service.

use std::path::PathBuf;

pub struct Struct {
	#[allow(dead_code)]
	workspace_root:Option<PathBuf>,
}

impl Struct {
	pub fn new() -> Self { Self { workspace_root:None } }

	pub fn with_root(Root:PathBuf) -> Self { Self { workspace_root:Some(Root) } }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
