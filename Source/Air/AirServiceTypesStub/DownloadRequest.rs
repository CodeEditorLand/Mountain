#![allow(non_snake_case)]

//! `DownloadFile` request DTO.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Struct {
	pub request_id:String,

	pub url:String,

	pub destination_path:String,

	pub checksum:String,

	pub headers:HashMap<String, String>,
}
