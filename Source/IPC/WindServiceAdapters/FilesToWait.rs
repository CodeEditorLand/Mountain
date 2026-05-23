
//! `--wait` payload: marker file URI plus the path list whose
//! close events the launcher is blocked on.

use serde::{Deserialize, Serialize};

use crate::IPC::WindServiceAdapters::FileToOpenOrCreate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub wait_marker_file_uri:String,

	pub paths:Vec<FileToOpenOrCreate::Struct>,
}
