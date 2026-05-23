
//! Per-service metric snapshot DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub name:String,

	pub count:u64,

	pub sum:f64,
}
