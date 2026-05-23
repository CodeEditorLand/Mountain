
//! Download-completion DTO returned by `DownloadUpdate` and
//! `DownloadFile`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub success:bool,

	pub file_path:String,

	pub file_size:u64,

	pub checksum:String,
}
