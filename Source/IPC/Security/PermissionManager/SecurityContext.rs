#![allow(non_snake_case)]

//! Security envelope used by `Manager::Struct::validate_permission`.
//! Carries the user id, roles, direct permissions, origin IP,
//! and request timestamp. `ipc_default` produces the loopback
//! "ipc-connection" context used for local IPC.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub user_id:String,

	pub roles:Vec<String>,

	pub permissions:Vec<String>,

	pub ip_address:String,

	pub timestamp:std::time::SystemTime,
}

impl Struct {

	pub fn new(user_id:String, roles:Vec<String>, permissions:Vec<String>, ip_address:String) -> Self {

		Self { user_id, roles, permissions, ip_address, timestamp:std::time::SystemTime::now() }
	}

	pub fn has_role(&self, role:&str) -> bool { self.roles.iter().any(|r| r == role) }

	pub fn has_permission(&self, permission:&str) -> bool { self.permissions.iter().any(|p| p == permission) }

	pub fn ipc_default() -> Self {

		Self {

			user_id:"ipc-connection".to_string(),

			roles:vec!["user".to_string()],

			permissions:vec![],

			ip_address:"127.0.0.1".to_string(),

			timestamp:std::time::SystemTime::now(),
		}
	}
}
