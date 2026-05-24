//! Result envelope returned by `TlsGenerateCert`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub hostname:String,

	pub success:bool,

	pub valid_until:String,

	pub message:String,
}
