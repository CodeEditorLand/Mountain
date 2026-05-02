#![allow(non_snake_case)]

//! `SearchFiles` response DTO.

use crate::Air::AirServiceTypesStub::FileResultProtoDTO;

#[derive(Debug, Clone)]
pub struct Struct {
	pub results:Vec<FileResultProtoDTO::Struct>,
	pub total_results:u32,
	pub error:String,
}
