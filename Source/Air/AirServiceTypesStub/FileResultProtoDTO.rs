#![allow(non_snake_case)]

//! Single file result inside `SearchFilesResponse`.

#[derive(Debug, Clone)]
pub struct Struct {
	pub path:String,
	pub size:u64,
	pub line:Option<u32>,
	pub content:Option<String>,
}
