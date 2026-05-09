#![allow(non_snake_case)]

//! Per-request security envelope - user identity, role list,
//! direct permissions, origin IP, and request timestamp. Used
//! by `Validator::Struct::ValidatePermission` to decide
//! whether to allow an operation.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub UserId:String,

	pub Roles:Vec<String>,

	pub Permissions:Vec<String>,

	pub IpAddress:String,

	pub Timestamp:SystemTime,
}
