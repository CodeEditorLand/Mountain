#![allow(non_snake_case)]

//! `AuthenticateUser` response DTO.

#[derive(Debug, Clone)]
pub struct Struct {
	pub success:bool,

	pub token:String,

	pub error:String,
}
