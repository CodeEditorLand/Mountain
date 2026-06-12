//! LSP-compatible position: zero-based line + character offset.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub line:u32,

	pub character:u32,
}
