//! Single workspace folder DTO.
use serde::{Deserialize, Serialize};

/// Workspace folder: identifies a workspace folder by URI and display name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub uri:String,

	pub name:String,
}
