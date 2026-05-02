#![allow(non_snake_case)]

//! `AuthenticateUser` request DTO.

#[derive(Debug, Clone)]
pub struct Struct {
	pub request_id:String,
	pub provider:String,
	pub credentials:serde_json::Value,
}
