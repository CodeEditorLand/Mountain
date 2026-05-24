//! `WorkspaceService::WithRoot`

use super::Struct;
use std::path::PathBuf;

pub fn Fn(Root:PathBuf) -> Struct { Self { workspace_root:Some(Root) } }
