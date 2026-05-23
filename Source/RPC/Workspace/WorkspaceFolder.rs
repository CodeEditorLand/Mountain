
//! Single workspace folder DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub uri:String,

	pub name:String,
}
