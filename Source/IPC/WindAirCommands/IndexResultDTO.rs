//! File-indexing summary DTO returned by `IndexFiles`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub success:bool,

	pub files_indexed:u32,

	pub total_size:u64,
}
