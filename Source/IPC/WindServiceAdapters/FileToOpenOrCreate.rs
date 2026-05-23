//! One file URI scheduled for open / create. Carried inside
//! `WindDesktopConfiguration::Struct::files_to_open_or_create`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub file_uri:String,
}
