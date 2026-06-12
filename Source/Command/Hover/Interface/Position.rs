//! LSP-compatible position: zero-based line + character offset.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data for struct.
pub struct Struct {
	pub line:u32,

	pub character:u32,
}
