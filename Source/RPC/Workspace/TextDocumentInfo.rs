//! Open-document metadata DTO.
use serde::{Deserialize, Serialize};

/// Open document metadata: carries URI, version, and language ID for an open
/// document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub uri:String,

	pub version:i32,

	pub language_id:String,
}
