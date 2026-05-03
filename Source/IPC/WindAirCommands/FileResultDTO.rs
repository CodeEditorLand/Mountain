#![allow(non_snake_case)]

//! Single search hit returned inside `SearchResultsDTO::Struct`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub path:String,
	pub size:u64,
	pub line:Option<u32>,
	pub content:Option<String>,
}
