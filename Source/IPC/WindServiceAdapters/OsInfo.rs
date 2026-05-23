//! Minimal OS slice surfaced to Wind - just the release / OS
//! identifier string.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub release:String,
}
