#![allow(non_snake_case)]

//! LSP-compatible position: zero-based line + character offset.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub line:u32,
	pub character:u32,
}
