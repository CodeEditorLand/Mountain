pub mod New;
pub mod WithRoot;

use std::path::PathBuf;

pub struct Struct {
	workspace_root:Option<PathBuf>,
}
