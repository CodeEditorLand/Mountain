//! Per-service metric snapshot DTO.
use serde::{Deserialize, Serialize};

/// Service metric snapshot: captures a named metric with count and sum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub name:String,

	pub count:u64,

	pub sum:f64,
}
