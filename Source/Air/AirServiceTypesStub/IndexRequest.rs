#![allow(non_snake_case)]

//! `IndexFiles` request DTO.

#[derive(Debug, Clone)]
pub struct Struct {
	pub request_id:String,

	pub paths:Vec<String>,

	pub recursive:bool,
}
