#![allow(non_snake_case)]

//! Vine gRPC connection info DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub service_name:String,

	pub endpoint:String,
}
