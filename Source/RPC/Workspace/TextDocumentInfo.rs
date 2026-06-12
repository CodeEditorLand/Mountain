//! Open-document metadata DTO.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub uri:String,

	pub version:i32,

	pub language_id:String,
}
