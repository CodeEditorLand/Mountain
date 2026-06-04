//! Update-availability DTO returned by `CheckForUpdates`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub update_available:bool,

	pub version:String,

	pub download_url:String,

	pub release_notes:String,
}
