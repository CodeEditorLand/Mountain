//! Command definition DTO.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub id:String,

	pub title:String,

	pub description:Option<String>,
}
