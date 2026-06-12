//! Command definition DTO.
use serde::{Deserialize, Serialize};

/// Command definition: identifies a registered command by id, title, and
/// optional description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub id:String,

	pub title:String,

	pub description:Option<String>,
}
