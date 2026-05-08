#![allow(non_snake_case)]

//! Authentication response DTO returned by `AuthenticateUser`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub success:bool,

	pub token:String,

	pub error:Option<String>,
}
