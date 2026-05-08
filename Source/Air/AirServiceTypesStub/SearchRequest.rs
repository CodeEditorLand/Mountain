#![allow(non_snake_case)]

//! `SearchFiles` request DTO.

#[derive(Debug, Clone)]
pub struct Struct {
	pub request_id:String,

	pub query:String,

	pub file_patterns:Vec<String>,

	pub max_results:u32,
}
